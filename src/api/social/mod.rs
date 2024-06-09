use actix_web::{
    http::header::{ContentDisposition, ContentType},
    web, HttpResponse, Scope,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, Pool};
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    game_organizer::GameOrganizerRequest,
    sql::{self, PlayerData},
    PlayerId,
};

pub fn social_scope() -> Scope {
    web::scope("/social")
        .route("/profile", web::get().to(get_info))
        .route("/profile/{id}", web::get().to(get_info_other))
        .route("/", web::post().to(add_player))
        .route("/{id}", web::delete().to(delete_player))
        .route("/possible_friends", web::get().to(get_possible_friends))
        .route("/download_fen/{game_id}", web::get().to(get_fen_file))
}

pub async fn get_info(id: AuthenticationToken, db_pool: web::Data<Pool<MySql>>) -> HttpResponse {
    println!("auth data");
    get_info_inner(id.id as u64, db_pool).await
}

async fn get_info_inner(id: u64, db_pool: web::Data<Pool<MySql>>) -> HttpResponse {
    let player_data = sql::get_player_data(&db_pool, id)
        .await
        .expect("Error when fetching data from dg: get_player_data");

    let friends = sql::get_friends(&db_pool, id)
        .await
        .expect("Error when fetching data from db: get_friends");

    let games = sql::get_player_games(&db_pool, id)
        .await
        .expect("Error when fetching games from db: get_games");

    let mut opponents_data: HashMap<u64, PlayerData> = HashMap::new();
    let mut games_json = Vec::new();
    for game in games {
        let (opponent_id, playing_color) = {
            if id == game.black as u64 {
                (game.white as u64, "white")
            } else {
                (game.black as u64, "black")
            }
        };
        let opponent_data = match opponents_data.get(&opponent_id) {
            Some(n) => n.clone(),
            None => {
                let opponent_data = sql::get_player_data(&db_pool, opponent_id).await.unwrap();
                opponents_data.insert(opponent_id, opponent_data.clone());
                opponent_data
            }
        };
        games_json.push(json!({
            "id": game.id,
            "playing": playing_color,
            "opponent": opponent_data,
            "num_of_moves": game.num_of_moves,
            "win": game.win,
            "singleplayer": game.singleplayer
        }));
    }

    HttpResponse::Ok().json(json!({
        "info": player_data,
        "friends": friends,
        "games": games_json,
        "id": id
    }))
}

pub async fn get_info_other(id: web::Path<i32>, db_pool: web::Data<Pool<MySql>>) -> HttpResponse {
    println!("path data");
    get_info_inner(id.into_inner() as u64, db_pool).await
}

#[derive(Debug, Deserialize)]
pub struct NewPlayer {
    id: PlayerId,
    request_id: Option<u32>,
    msg_type: PlayerRequestType,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum PlayerRequestType {
    New,
    Accept,
    Reject,
    DeleteNotification(u64),
}

pub async fn add_player(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
    data: web::Json<NewPlayer>,
    game_organizer: web::Data<mpsc::Sender<GameOrganizerRequest>>,
) -> HttpResponse {
    if sql::get_friends(&db_pool, id.id as u64)
        .await
        .expect("Error when fetching data from db: get_friends")
        .into_iter()
        .any(|p| p.id == data.id as i32)
    {
        return HttpResponse::BadRequest().json(json!({"reason": "Player is already friend"}));
    } else if id.id == data.id {
        if let PlayerRequestType::DeleteNotification(_) = data.msg_type {
        } else {
            return HttpResponse::BadRequest().json(json!({"reason": "You cant add yourself"}));
        }
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
        PlayerRequestType::DeleteNotification(req_id) => {
            println!("wooooowwww");
            let _ = game_organizer
                .send(GameOrganizerRequest::DeleteNotification(id.id, req_id))
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
    println!("entered");
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

pub async fn get_fen_file(
    id: AuthenticationToken,
    db_pool: web::Data<Pool<MySql>>,
    game_id: web::Path<u64>,
) -> HttpResponse {
    let game_id = game_id.into_inner();
    let res = match sqlx::query!(
        "SELECT game_file_uuid FROM Games WHERE id=? AND (white=? OR black=?)",
        game_id,
        id.id as u64,
        id.id as u64,
    )
    .fetch_one(db_pool.as_ref())
    .await
    {
        Ok(g) => g,
        Err(_) => return HttpResponse::BadRequest().body("Game id not found"),
    };

    let file_content =
        std::fs::read_to_string(format!("/games/{}.pgn", res.game_file_uuid)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .insert_header(ContentDisposition::attachment(format!(
            "ChezzGame{}.pgn",
            game_id
        )))
        .body(file_content)
}
