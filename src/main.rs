//! # Near Earth Object API investigator.
//! Basic website design is to have a way of searching NEOs by date range, then picking more
//! info for a specific NEO.
//!

mod neo_structs;

use actix_web::{get, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use reqwest::Client;
use crate::neo_structs::NeoFeed;
use serde::{Deserialize, Serialize};
use handlebars::{DirectorySourceOptions, Handlebars};

#[derive(Deserialize, Serialize)]
struct NeoDateSearch {
    date: String,
}

mod neo_feed {
    use actix_web::{post, web, HttpResponse, Responder};
    use handlebars::Handlebars;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use crate::NeoDateSearch;
    use crate::neo_structs::NeoFeed;

    #[derive(Deserialize, Serialize)]
    struct NeoDetails {
        name: String,
        size: String,
        velocity: String,
        distance: String,
        time: String,
        hazardous: bool,
        link: String,
    }
    #[derive(Deserialize, Serialize)]
    struct NeoVec {
        neos: Vec<NeoDetails>,
    }

    impl NeoFeed {
        fn to_neo_vec(self) -> Vec<NeoDetails> {
            let mut result_vec: Vec<NeoDetails> = Vec::new();
            for day in self.near_earth_objects.days {
                let day_string = day.0;
                for neo in day.1 {
                    let n = NeoDetails {
                        name: neo.name,
                        size: neo.estimated_diameter.kilometers.estimated_diameter_max.to_string(),
                        velocity: neo.close_approach_data.get(0).unwrap().relative_velocity.kilometers_per_hour.to_string(),
                        distance: neo.close_approach_data.get(0).unwrap().miss_distance.kilometers.to_string(),
                        time: neo.close_approach_data.get(0).unwrap().close_approach_date_full.to_string(),
                        hazardous: neo.is_potentially_hazardous_asteroid,
                        link: format!("/neo/{}", neo.neo_reference_id),
                    };
                    result_vec.push(n);
                }
            }
            result_vec
        }
    }
    #[post("/date/{date}")]
    // todo - make it so that the form submission kicks to a date url so that it's easier to pivot from
    // NEO look up dates.
    // date format = 2015-09-07
    pub async fn neo_feed_page(path: web::Path<(String)>, handlebars: web::Data<Handlebars<'_>>) -> impl Responder {
        let dates = path.into_inner();
        let api_call = format!("https://api.nasa.gov/neo/rest/v1/feed?start_date={}&end_date={}&api_key=DEMO_KEY", dates, dates);
        let response = Client::new().get(api_call).send().await.unwrap();
        let neo_data = response.json::<NeoFeed>().await.unwrap();

        let feed = NeoVec {
            neos: neo_data.to_neo_vec()
        };
        let rendered = handlebars.render("NEO_feed", &feed).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

mod neo_lookup {
    use actix_web::{get, web, HttpResponse, Responder};
    use handlebars::Handlebars;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use crate::neo_structs::NeoLookup;

    #[derive(Deserialize, Serialize, Debug)]
    struct NeoDetailsForLookupPage {
        neo_name: String,
        diameter: String,
        hazardous: bool,
        eccentricity: String,
        inclination: String,
        close_approach: Vec<ApproachDataForLookupPage>,
    }

    #[derive(Deserialize, Serialize, Debug)]
    struct ApproachDataForLookupPage {
        date: String,
        velocity: String,
        miss_distance: String,
        orbiting_body: String,
    }

    impl NeoLookup {
        fn to_hbs_format(self) -> NeoDetailsForLookupPage {
            let mut close_approaches: Vec<ApproachDataForLookupPage> = Vec::new();
            for approach in self.close_approach_data {
                let a = ApproachDataForLookupPage {
                    date: approach.close_approach_date,
                    velocity: approach.relative_velocity.kilometers_per_hour.to_string(),
                    miss_distance: approach.miss_distance.kilometers.to_string(),
                    orbiting_body: approach.orbiting_body.to_string(),
                };
                close_approaches.push(a);
            }

            NeoDetailsForLookupPage {
                neo_name: self.name,
                diameter: self.estimated_diameter.kilometers.estimated_diameter_max.to_string(),
                hazardous: self.is_potentially_hazardous_asteroid,
                eccentricity: self.orbital_data.eccentricity.to_string(),
                inclination: self.orbital_data.inclination.to_string(),
                close_approach: close_approaches,
            }
        }
    }
    #[get("/neo/{neo_id}")]
    pub async fn get_single_neo(path: web::Path<(u32)>, handlebars: web::Data<Handlebars<'_>>) -> impl Responder {
        let api_call = format!("https://api.nasa.gov/neo/rest/v1/neo/{}?api_key=DEMO_KEY", path.into_inner());
        let response = Client::new().get(api_call).send().await.unwrap();
        let neo_data = response.json::<NeoLookup>().await.unwrap();

        let feed = neo_data.to_hbs_format();
        println!("{:?}", &feed);
        let rendered = handlebars.render("NEO_lookup", &feed).unwrap();
        HttpResponse::Ok().body(rendered)
    }
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

// todo - dates can't be in the future.

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
        )
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    HttpServer::new(move || {
        App::new()
            .app_data(handlebars_ref.clone())
            .service(index)
            .service(neo_feed::neo_feed_page)
            .service(neo_lookup::get_single_neo)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
