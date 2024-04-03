use futures::future::Future;
use serde::Serialize;
use sqlx::{MySql, Pool};

use crate::PlayerId;

#[derive(Debug, Clone, Serialize)]
pub struct PlayerData {
    pub id: i32,
    pub username: String,
    pub country: Option<String>,
}

impl PlayerData {
    pub fn singleplayer(player_id: PlayerId) -> Self {
        Self {
            id: player_id as i32,
            username: "Singleplayer".into(),
            country: None,
        }
    }
}

pub fn get_player_data(
    db_pool: &Pool<MySql>,
    player_id: u64,
) -> impl Future<Output = Result<PlayerData, sqlx::Error>> + '_ {
    sqlx::query_as!(
        PlayerData,
        "SELECT id, username, country from User where id=?",
        player_id as u64,
    )
    .fetch_one(db_pool)
}

pub async fn get_friends(
    db_pool: &Pool<MySql>,
    player_id: u64,
) -> Result<Vec<PlayerData>, sqlx::Error> {
    sqlx::query_as!(
        PlayerData,
        "SELECT id, username, country FROM User
        WHERE id in (
            SELECT friend2 friend FROM Friends
            WHERE friend1=?
            UNION
            SELECT friend1 friend FROM Friends
            WHERE friend2=?
        );",
        player_id as u64,
        player_id as u64,
    )
    .fetch_all(db_pool)
    .await
}
