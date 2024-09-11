mod structs;

use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use handlebars::Handlebars;



/// Pull the data from the NASA api.
async fn get_nasa_api(req: web::Json<structs::Root>) -> Result<String> {
    Ok(format!("{:?}", req.links))

}


/// Function to call and serve handlebars enabled html index page
///
#[get("/")]
async fn index(hb: web::Data<Handlebars<'_>>) -> impl Responder {
    todo!()
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
