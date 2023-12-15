use actix_web::{web, HttpResponse, Scope};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::extractors::authentication_token::{AuthenticationToken, Claims};

mod login;
mod register;

#[derive(Debug, Serialize, Deserialize)]
struct AccessToken {
    access_token: String,
    id: i32,
}

async fn encode_token(id: usize, secret: web::Data<String>) -> String {
    let exp = (Utc::now() + Duration::days(30)).timestamp() as usize;
    let claims = Claims { id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_str().as_ref()),
    )
    .expect("Couldn't make access token");
    token
}

pub async fn test(id: AuthenticationToken) -> HttpResponse {
    println!("Auth token: {:?}", id);
    HttpResponse::Ok().into()
}

pub fn login_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::post().to(login::login))
        .route("/test", web::get().to(test))
        .route("/register", web::post().to(register::register))
}
