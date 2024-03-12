use actix_web::{web, HttpResponse, Scope};
use futures::task::UnsafeFutureObj;
use serde::Deserialize;
use serde_json::json;
use sqlx::{MySql, Pool};

use crate::{
    extractors::authentication_token::AuthenticationToken,
    sql::{self, PlayerData},
    PlayerId,
};

pub fn social_scope() -> Scope {
    web::scope("/social")
        .route("/", web::get().to(get_info))
        .route("/", web::post().to(add_player))
        .route("/{id}", web::delete().to(delete_player))
}

pub async fn get_info(id: AuthenticationToken, db_pool: web::Data<Pool<MySql>>) -> HttpResponse {
    let info = sql::get_friends(&db_pool, id.id as u64)
        .await
        .expect("Error when fetching data from db: get_friends");
    println!("info: {info:?}");

    HttpResponse::Ok().json(json!({"info": info, "id": id}))
}

#[derive(Deserialize)]
pub struct NewPlayer {
    id: PlayerId,
}

pub async fn add_player(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
    data: web::Json<NewPlayer>,
) -> HttpResponse {
    if sql::get_friends(&db_pool, id.id as u64)
        .await
        .expect("Error when fetching data from db: get_friends")
        .into_iter()
        .any(|p| p.id == data.id as i32)
    {
        return HttpResponse::BadRequest().json(json!({"reason": "Player is already friend"}));
    } else if id.id == data.id {
        return HttpResponse::BadRequest().json(json!({"reason": "You cant add yourself"}));
    }
    let res = sqlx::query!(
        "INSERT into Friends(friend1, friend2) values (?, ?)",
        id.id as u64,
        data.id as u64
    )
    .execute(db_pool.get_ref())
    .await;
    match res {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::BadRequest().body("u suck"),
    }
}

pub async fn delete_player(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
    data: web::Path<u64>,
) -> HttpResponse {
    let other_id = data.into_inner() as u64;
    println!("id {}, oth id: {other_id}", id.id);
    if id.id as u64 == other_id {
        return HttpResponse::BadRequest().json(json!({"reason": "You cant add yourself"}));
    }

    match sqlx::query!(
        "DELETE from Friends where friend1=? and friend2=? or friend1=? and friend2=?",
        id.id as u64,
        other_id,
        other_id,
        id.id as u64,
    )
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::BadRequest().into(),
    }
}
