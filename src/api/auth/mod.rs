use actix_web::{web, HttpResponse, Scope};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, Pool};

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

pub async fn get_username(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
) -> HttpResponse {
    struct Username {
        username: String,
    }
    let Username { username } = sqlx::query_as!(
        Username,
        "SELECT username
        FROM User
        WHERE id = ?",
        id.id as u64
    )
    .fetch_one(db_pool.get_ref())
    .await
    .expect(format!("DB err, there should be a user with id: {}", id.id).as_str());
    HttpResponse::Ok().json(json!({"username": username}))
}

pub fn login_scope() -> Scope {
    web::scope("/auth")
        .route("/login", web::post().to(login::login))
        .route("/", web::get().to(get_username))
        .route("/register", web::post().to(register::register))
}
