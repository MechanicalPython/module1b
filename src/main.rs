//! # Near Earth Object API investigator.
//! Basic website design is to have a way of searching NEOs by date range, then picking more
//! info for a specific NEO.
//! ## How to use.
//! NEO feed = a list of NEOs given a date (or date range)
//! NEO lookup = details of a single NEO.

mod neo_structs;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use actix_session::{Session, SessionMiddleware, storage::CookieSessionStore};
use actix_session::config::{BrowserSession, CookieContentSecurity};
use actix_web::cookie::{Key, SameSite};
use handlebars::{DirectorySourceOptions, Handlebars};
use serde::{Deserialize, Serialize};
use crate::neo_structs::{NeoFeed, NeoLookup};

mod neo_feed {
    use std::fs::{read_to_string};
    use actix_session::Session;
    use actix_web::{get, web, HttpResponse, Responder};
    use handlebars::Handlebars;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use crate::neo_structs::{NeoFeed};
    use crate::TopTrumpsCounter;

    #[derive(Deserialize, Serialize)]
    struct NeoFeedDetails {
        name: String,
        size: i32,
        velocity: i32,
        distance: i32,
        time: String,
        hazardous: bool,
        reference_id: String,
    }

    #[derive(Deserialize, Serialize)]
    struct NeoFeedDetailsVec {
        neos: Vec<NeoFeedDetails>,
        fastest: i64,
        closest: i64,
        neos_seen: i64
    }

    impl NeoFeed {
        // in neo.close_approach_data, it will be a vec of length 1 always when getting feed data.
        fn into_neo_feed_details(self) -> Vec<NeoFeedDetails> {
            let mut result_vec: Vec<NeoFeedDetails> = Vec::new();
            for day in self.near_earth_objects.days {
                let _day_string = day.0;
                for neo in day.1 {
                    let n = NeoFeedDetails {
                        name: neo.name,
                        size: neo.estimated_diameter.meters.estimated_diameter_max as i32,
                        velocity: neo.close_approach_data.first().unwrap().relative_velocity.kilometers_per_hour as i32,
                        distance: neo.close_approach_data.first().unwrap().miss_distance.kilometers as i32,
                        time: neo.close_approach_data.first().unwrap().close_approach_date_full.to_string(),
                        hazardous: neo.is_potentially_hazardous_asteroid,
                        reference_id: neo.neo_reference_id,
                    };
                    result_vec.push(n);
                }
            }
            result_vec
        }
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct QueryResponse {
        neo_search: String,
    }

    // /date?neo_search=yyyy-mm-dd
    // date format = 2015-09-07
    #[get("/date")]
    pub async fn neo_feed_page(path: web::Query<QueryResponse>,
                               handlebars: web::Data<Handlebars<'_>>,
                               session: Session) -> impl Responder {
        let api_key = read_to_string("api_key").unwrap_or("DEMO_KEY".to_string());
        let api_call = format!("https://api.nasa.gov/neo/rest/v1/feed?start_date={}&end_date={}&api_key={}",
                               path.neo_search, path.neo_search, api_key);
        let response = Client::new().get(api_call).send().await.unwrap();
        let neo_data = response.json::<NeoFeed>().await.unwrap();  // Error if struct type doesn't match json type.

        let current_top_trumps = session.get::<TopTrumpsCounter>("top_trumps").unwrap().unwrap();

        let new_top_trumps = current_top_trumps.update_count_for_feed(&neo_data);
        session.insert("top_trumps", &new_top_trumps).unwrap();

        let feed = NeoFeedDetailsVec {
            neos: neo_data.into_neo_feed_details(),
            fastest: new_top_trumps.fastest,
            closest: new_top_trumps.closest,
            neos_seen: new_top_trumps.total_neos_seen,
        };

        let rendered = handlebars.render("NEO_feed", &feed).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

mod neo_lookup {
    use std::fs::read_to_string;
    use actix_session::Session;
    use actix_web::{get, web, HttpResponse, Responder};
    use handlebars::Handlebars;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use crate::neo_structs::NeoLookup;
    use crate::TopTrumpsCounter;

    #[derive(Deserialize, Serialize, Debug)]
    struct NeoLookupForHTML {
        neo_name: String,
        diameter: i32,
        hazardous: bool,
        eccentricity: String,
        inclination: String,
        close_approach: Vec<NeoApproachData>,
        fastest: i64,
        closest: i64,
        neos_seen: i64,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct NeoApproachData {
        date: String,
        velocity: i32,
        miss_distance: i32,
        orbiting_body: String,
    }

    impl NeoLookup {
        fn into_hbs_format(self, stats: TopTrumpsCounter) -> NeoLookupForHTML {
            let mut close_approaches: Vec<NeoApproachData> = Vec::new();
            for approach in self.close_approach_data {
                let a = NeoApproachData {
                    date: approach.close_approach_date,
                    velocity: approach.relative_velocity.kilometers_per_hour as i32,
                    miss_distance: approach.miss_distance.kilometers as i32,
                    orbiting_body: approach.orbiting_body.to_string(),
                };
                close_approaches.push(a);
            }

            NeoLookupForHTML {
                neo_name: self.name,
                diameter: self.estimated_diameter.meters.estimated_diameter_max as i32,
                hazardous: self.is_potentially_hazardous_asteroid,
                eccentricity: self.orbital_data.eccentricity.to_string(),
                inclination: self.orbital_data.inclination.to_string(),
                close_approach: close_approaches,
                fastest: stats.fastest,
                closest: stats.closest,
                neos_seen: stats.total_neos_seen
            }
        }
    }
    #[get("/neo/{neo_id}")]
    pub async fn get_single_neo(path: web::Path<u32>,
                                handlebars: web::Data<Handlebars<'_>>,
                                session: Session) -> impl Responder {

        let neo_lookup = Client::new().get(
            format!("https://api.nasa.gov/neo/rest/v1/neo/{}?api_key={}",
                    path.into_inner(),
                    read_to_string("api_key").unwrap_or("DEMO_KEY".to_string())))
            .send().await.unwrap()
            .json::<NeoLookup>().await.unwrap();

        let current_top_trumps = session.get::<TopTrumpsCounter>("top_trumps").unwrap().unwrap();

        let new_top_trumps = current_top_trumps.update_count_for_lookup(&neo_lookup);
        session.insert("top_trumps", &new_top_trumps).unwrap();

        let feed = neo_lookup.into_hbs_format(new_top_trumps);
        let rendered = handlebars.render("NEO_lookup", &feed, ).unwrap();
        dbg!(&rendered);
        HttpResponse::Ok().body(rendered)
    }
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct TopTrumpsCounter {
    fastest: i64,
    closest: i64,
    total_neos_seen: i64,
}

impl TopTrumpsCounter {
    fn default() -> Self {
        Self{
            fastest: 0,
            // required to make the comparison code below work.
            closest: i64::MAX,
            total_neos_seen: 0,
        }
    }

    fn update_count_for_lookup(self, lookup: &NeoLookup) -> Self {
        let mut fastest = self.fastest;
        let mut closest = self.closest;

        for approach in &lookup.close_approach_data {
            if approach.relative_velocity.kilometers_per_hour as i64 > fastest {
                fastest = approach.relative_velocity.kilometers_per_hour as i64;
            }
            // Needs smaller of two values.
            if closest > approach.relative_velocity.kilometers_per_hour as i64 {
                closest = approach.miss_distance.kilometers as i64;
            }
        }
        Self{
            fastest,
            closest,
            total_neos_seen: self.total_neos_seen,
        }
    }

    fn update_count_for_feed(self, feed: &NeoFeed) -> Self {
        let mut fastest = self.fastest;
        let mut closest = self.closest;
        for neos in feed.near_earth_objects.days.iter()
            .map(| neo| neo.1) {
            for neo in neos {
                if neo.close_approach_data.first().unwrap().relative_velocity.kilometers_per_hour as i64 > fastest {
                    fastest = neo.close_approach_data.first().unwrap().relative_velocity.kilometers_per_hour as i64
                }
                // needs the smaller of the values.
                if closest > neo.close_approach_data.first().unwrap().miss_distance.kilometers as i64 {
                    closest = neo.close_approach_data.first().unwrap().miss_distance.kilometers as i64
                }
            }
        };
        Self {
            fastest,
            closest,
            total_neos_seen: { self.total_neos_seen + feed.element_count },
        }
    }
}

fn session_cookie_middleware() -> SessionMiddleware<CookieSessionStore> {
    SessionMiddleware::builder(
        CookieSessionStore::default(), Key::from(&[0; 64])
    )
        .cookie_name(String::from("top_trumps_store"))
        .cookie_secure(false)
        .session_lifecycle(BrowserSession::default())
        .cookie_same_site(SameSite::Strict)
        .cookie_content_security(CookieContentSecurity::Signed)
        .cookie_http_only(true)
        .build()
}

#[get("/")]
async fn index(session: Session) -> impl Responder {
    session.insert("top_trumps", TopTrumpsCounter::default()).unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(
            "./static",
            DirectorySourceOptions {
                tpl_extension: ".html".to_owned(),
                hidden: false,
                temporary: false,
            },
        ).unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .wrap(session_cookie_middleware())
            .app_data(handlebars_ref.clone())
            .service(index)
            .service(neo_feed::neo_feed_page)
            .service(neo_lookup::get_single_neo)
            .service(Files::new("/static", "./static"))  // No need to enable file listing unless you actually need want it to be enabled
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

