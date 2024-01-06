use sqlx::{MySql, Pool};

pub async fn get_player_data(
    db_pool: &Pool<MySql>,
    player_id: u64,
) -> Result<PlayerData, sqlx::Error> {
    sqlx::query_as!(
        PlayerData,
        "SELECT id, username, country from User where id=?",
        player_id as u64,
    )
    .fetch_one(db_pool)
    .await
}
pub struct PlayerData {
    id: i32,
    username: String,
    country: Option<String>,
}
