use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use sqlx::{MySql, Pool};

use super::{encode_token, AccessToken};

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    username: String,
    password: String,
}

struct UserIdSelect {
    id: i32,
}

pub async fn register(
    new_user_data: web::Json<RegisterBody>,
    secret: web::Data<String>,
    db_pool: web::Data<Pool<MySql>>,
) -> HttpResponse {
    match sqlx::query!(
        "INSERT into User(username, password)
        values (?, ?);",
        new_user_data.username,
        new_user_data.password
    )
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => {
            let user = sqlx::query_as!(
                UserIdSelect,
                "SELECT id
                FROM User
                WHERE username = ?",
                new_user_data.username
            )
            .fetch_one(db_pool.get_ref())
            .await
            .expect("User should be in database, because I just inserted it");

            let token = encode_token(user.id as usize, secret).await;

            HttpResponse::Ok().json(AccessToken {
                id: user.id,
                access_token: token,
            })
        }
        Err(_) => HttpResponse::BadRequest().json(json!({"reason": "Couldnt create a new user"})),
    }
}
