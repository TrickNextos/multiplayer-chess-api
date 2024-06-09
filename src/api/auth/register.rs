use actix_web::{
    cookie::{time::OffsetDateTime, CookieBuilder},
    web, HttpResponse,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::{MySql, Pool};

use super::{calculate_hash, encode_token};

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    username: String,
    password: String,
}

struct UserIdSelect {
    id: i32,
}

fn check_password_req(pass: &str) -> Result<(), String> {
    if pass.len() < 8 || pass.len() > 24 {
        return Err("Password must be between 8 and 24 characters long".into());
    }
    // let requirements = [
    //     |p: char| p.is_numeric(),
    //     |p: char| p.is_ascii_lowercase(),
    //     |p: char| p.is_ascii_uppercase(),
    //     |p: char| //checks for special characters by looking into at ascii values
    //         [33..48, 58..65, 91..97, 123..127]
    //             .iter()
    //             .any(|range| range.contains(&(p as u8))),
    // ];
    // let req_messages = [
    //     "number",
    //     "lowercase character",
    //     "uppercase character",
    //     "symbol",
    // ];
    //
    // let mut req_are_met = [false; 4];
    // for (i, req) in requirements.iter().enumerate() {
    //     for c in pass.chars() {
    //         if req(c) {
    //             req_are_met[i] = true;
    //             break;
    //         }
    //     }
    // }
    //
    // if req_are_met.iter().all(|t| *t) {
    //     Ok(())
    // } else {
    //     let mut reasons = "Password must also include a: <ul>".to_owned();
    //     for i in 0..4 {
    //         if !req_are_met[i] {
    //             reasons = reasons + "<li>" + req_messages[i] + "</li>";
    //         }
    //     }
    //     Err(reasons + "</ul>")
    // }
    Ok(())
}

pub async fn register(
    new_user_data: web::Json<RegisterBody>,
    secret: web::Data<String>,
    db_pool: web::Data<Pool<MySql>>,
) -> HttpResponse {
    println!("got response");
    if new_user_data.username.len() < 4 || new_user_data.username.len() > 18 {
        return HttpResponse::BadRequest()
            .json(json!({"reason": "Bad username", "description": "Username must be between 4 and 18 characters long"}));
    }
    for c in new_user_data.username.chars() {
        if !c.is_ascii() || c.is_whitespace() {
            return HttpResponse::BadRequest()
            .json(json!({"reason": "Bad username", "description": "Username must contain only ascii charaters"}));
        }
    }

    let user_exists = sqlx::query_as!(
        UserIdSelect,
        "SELECT id
        FROM User
        WHERE username = ?",
        new_user_data.username
    )
    .fetch_optional(db_pool.get_ref())
    .await;
    if let Ok(Some(_)) = user_exists {
        return HttpResponse::BadRequest()
            .json(json!({"reason": "Bad username", "description": "Username alreay taken"}));
    } else if let Err(err) = user_exists {
        return HttpResponse::InternalServerError().body(format!("db error: {err}"));
    }

    if let Err(reason) = check_password_req(&new_user_data.password) {
        return HttpResponse::BadRequest()
            .json(json!({"reason": "Bad password", "description": reason}));
    }

    match sqlx::query!(
        "INSERT into User(username, password)
        values (?, ?);",
        new_user_data.username,
        calculate_hash(&new_user_data.password)
    )
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => {
            let user_id = sqlx::query_as!(
                UserIdSelect,
                "SELECT id
                FROM User
                WHERE username = ?",
                new_user_data.username
            )
            .fetch_one(db_pool.get_ref())
            .await
            .expect("User should be in database, because I just inserted it");

            let token = encode_token(user_id.id as usize, secret).await;

            let cookie =
                CookieBuilder::new(crate::extractors::authentication_token::COOKIE_NAME, token)
                    .http_only(true)
                    .path("/")
                    .expires(
                        OffsetDateTime::from_unix_timestamp(
                            (Utc::now() + Duration::days(30)).timestamp(),
                        )
                        .unwrap(),
                    )
                    .finish();
            HttpResponse::Ok().cookie(cookie).json(json!({
                "id": user_id.id,
            }))
        }
        Err(err) => HttpResponse::InternalServerError().body(format!("db error: {err}")),
    }
}
