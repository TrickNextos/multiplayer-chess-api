use actix_web::{web, HttpResponse, Scope};
use serde_json::json;
use sqlx::{MySql, Pool};
use serde::Deserialize;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    sql::{self, PlayerData}, PlayerId,
};

pub fn social_scope() -> Scope {
    web::scope("/social")
        .route("/", web::get().to(get_info))
        .route("/add", web::post().to(add_player))
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

pub async fn add_player(id: AuthenticationToken, db_pool: web::Data<Pool<MySql>>, data: web::Json<NewPlayer>) -> HttpResponse {
    if sql::get_friends(&db_pool, id.id as u64)
        .await
        .expect("Error when fetching data from db: get_friends")
        .into_iter()
        .any(|p| p.id == data.id as i32){
        return HttpResponse::BadRequest().json(json!({"reason": "Player is already friend"}));
    }
    let res = sqlx::query!("INSERT into Friends(friend1, friend2) values (?, ?)", id.id as u64, data.id as u64)
        .execute(db_pool.get_ref())
        .await;
    match res {
        Ok(_) => HttpResponse::Ok().into(),
        Err(_) => HttpResponse::BadRequest().body("u suck"),
        
    }
}
