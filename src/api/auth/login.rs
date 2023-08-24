use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, Pool};

use super::encode_token;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginBody {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessToken {
    access_token: String,
    id: i32,
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
    let token = encode_token(203512 as usize, secret).await;

    let user = match sqlx::query_as!(
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
            return HttpResponse::BadRequest().json(json!({"reason": "No user"}))
        }
        Err(err) => panic!("Unexpected error, {err}"),
    };

    if user.password == credentials.password {
        HttpResponse::Ok().json(AccessToken {
            access_token: token,
            id: user.id,
        })
    } else {
        HttpResponse::BadRequest().json(json!({"reason": "Wrong password"}))
    }
}
