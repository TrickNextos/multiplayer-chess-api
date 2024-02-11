use futures::future::Future;
use serde::Serialize;
use sqlx::{MySql, Pool};

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

#[derive(Debug, Serialize)]
pub struct PlayerData {
    pub id: i32,
    pub username: String,
    pub country: Option<String>,
}
