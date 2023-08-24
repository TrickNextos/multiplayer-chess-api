use actix_web::{HttpRequest, HttpResponse};

pub async fn healthcheck(request: HttpRequest) -> HttpResponse {
    println!("{:?}", request);
    HttpResponse::Ok().body("Healthcheck OK")
}
