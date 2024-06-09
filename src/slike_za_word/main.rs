use actix_web::{web, App, HttpServer, HttpResponse};

pub async fn healthcheck() -> HttpResponse {
    HttpResponse::Ok().body("Healthcheck OK")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("server starting");
    
    HttpServer::new(move || App::new().route("/healthcheck", web::get().to(healthcheck)))
    .bind(("0.0.0.0", 5678))?
    .run()
    .await
}



