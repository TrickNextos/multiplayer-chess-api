use actix_web::{cookie::Cookie, HttpRequest, HttpResponse};

pub async fn healthcheck(request: HttpRequest) -> HttpResponse {
    println!("{:?}", request);
    let c = Cookie::new("test", "Woooow it works");
    HttpResponse::Ok().cookie(c).body("Healthcheck OK")
}
