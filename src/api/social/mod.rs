use actix_web::{web, HttpResponse, Scope};
use serde::Deserialize;
use serde_json::json;
use sqlx::{MySql, Pool};
use tokio::sync::mpsc;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    game_organizer::GameOrganizerRequest,
    sql::{self, PlayerData},
    PlayerId,
};

pub fn social_scope() -> Scope {
    web::scope("/social")
        .route("/", web::get().to(get_info))
        .route("/", web::post().to(add_player))
        .route("/{id}", web::delete().to(delete_player))
        .route("/possible_friends", web::get().to(get_possible_friends))
}

pub async fn get_info(id: AuthenticationToken, db_pool: web::Data<Pool<MySql>>) -> HttpResponse {
    let info = sql::get_friends(&db_pool, id.id as u64)
        .await
        .expect("Error when fetching data from db: get_friends");
    println!("info: {info:?}");

    HttpResponse::Ok().json(json!({"info": info, "id": id}))
}

#[derive(Debug, Deserialize)]
pub struct NewPlayer {
    id: PlayerId,
    request_id: Option<u32>,
    msg_type: PlayerRequestType,
}

#[derive(Debug, Deserialize)]
enum PlayerRequestType {
    New,
    Accept,
    Reject,
}

pub async fn add_player(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
    data: web::Json<NewPlayer>,
    game_organizer: web::Data<mpsc::Sender<GameOrganizerRequest>>,
) -> HttpResponse {
    println!("INFOOO: {data:?}");
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

    match data.msg_type {
        PlayerRequestType::New => {
            println!("new friend req");
            let request_id = rand::random();
            let _ = game_organizer
                .send(GameOrganizerRequest::FriendNew(request_id, id.id, data.id))
                .await;

            HttpResponse::Ok().json(json!({"request_id": request_id}))
        }
        PlayerRequestType::Accept => {
            let request_id = match data.request_id {
                Some(n) => n,
                None => {
                    return HttpResponse::BadRequest()
                        .json(json!({"reason": "Request id wasnt specified"}))
                }
            };
            let _ = game_organizer
                .send(GameOrganizerRequest::FriendAccept(
                    request_id, data.id, id.id,
                ))
                .await;
            HttpResponse::Ok().into()
        }
        PlayerRequestType::Reject => {
            let request_id = match data.request_id {
                Some(n) => n,
                None => {
                    return HttpResponse::BadRequest()
                        .json(json!({"reason": "Request id wasnt specified"}))
                }
            };

            let _ = game_organizer
                .send(GameOrganizerRequest::FriendReject(
                    request_id, data.id, id.id,
                ))
                .await;
            HttpResponse::Ok().into()
        }
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

pub async fn get_possible_friends(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
) -> HttpResponse {
    match sqlx::query_as!(
        PlayerData,
        "SELECT id, username, country from User
        WHERE  id not in (SELECT friend1 from Friends where friend2=? UNION SELECT friend2 from Friends where friend1=?)
        and id != ?",
        id.id as u64,
        id.id as u64,
        id.id as u64,
    )
        .fetch_all(db_pool.get_ref())
        .await {
            Ok(res) => HttpResponse::Ok().json(json!({
                "id": id.id,
                "info": res,
            })),
                Err(_) => HttpResponse::BadRequest().json(json!({"reason": "db fail"})),
        }
}
