use actix_web::{
    cookie::{time::OffsetDateTime, CookieBuilder},
    web, HttpResponse,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, Pool};

use super::{calculate_hash, encode_token};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    username: String,
    password: String,
}

struct UserSelect {
    id: i32,
    password: String,
}

pub async fn login(
    credentials: web::Json<LoginBody>,
    secret: web::Data<String>,
    db_pool: web::Data<Pool<MySql>>,
) -> HttpResponse {
    let user: UserSelect = match sqlx::query_as!(
        UserSelect,
        "SELECT id, password
        FROM User
        WHERE username = ?",
        credentials.username
    )
    .fetch_one(db_pool.get_ref())
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return HttpResponse::BadRequest()
                .json(json!({"reason": "Bad username", "description": "User not found"}))
        }
        Err(err) => panic!("Unexpected error, {}", err),
    };
    let token = encode_token(user.id as usize, secret).await;
    let cookie = CookieBuilder::new(crate::extractors::authentication_token::COOKIE_NAME, token)
        .http_only(true)
        .path("/")
        .expires(
            OffsetDateTime::from_unix_timestamp((Utc::now() + Duration::days(30)).timestamp())
                .unwrap(),
        )
        .finish();

    if user.password == calculate_hash(&credentials.password).to_string() {
        HttpResponse::Ok().cookie(cookie).json(json!({
            "id": user.id,
        }))
    } else {
        HttpResponse::BadRequest()
            .json(json!({"reason": "Bad password", "description": "Wrong password"}))
    }
}
