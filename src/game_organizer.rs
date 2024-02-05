use serde::Serialize;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::chess_logic::{ChessGame, Position};
use serde_json::json;
use sqlx::{MySql, Pool};

use crate::{api::game_ws::ChessEnd, GameId, PlayerId, WsMessageOutgoing};

#[derive(Debug)]
pub struct GameOrganizer {
    current_games: HashMap<GameId, ChessGame>,
    waiting_player: Option<PlayerId>,
    current_players: HashMap<PlayerId, mpsc::Sender<String>>,

    db_pool: Pool<MySql>,
}

impl GameOrganizer {
    pub fn new(db_pool: Pool<MySql>) -> mpsc::Sender<GameOrganizerRequest> {
        let mut instance = Self {
            db_pool,
            current_games: Default::default(),
            waiting_player: Default::default(),
            current_players: Default::default(),
        };

        let (tx, mut rx) = mpsc::channel::<GameOrganizerRequest>(32);

        actix_rt::spawn(async move {
            while let Some(msg) = rx.recv().await {
                dbg!(msg.clone());
                use GameOrganizerRequest::*;
                match msg {
                    Move(p_id, g_id, from, to) => instance.r#move(p_id, g_id, from, to).await,
                    Chat(p_id, g_id, text) => instance.chat(p_id, g_id, text).await,
                    End(p_id, g_id, reason) => instance.end(p_id, g_id, reason),
                    NewGame(p_id) => instance.new_game(p_id).await,
                    Connect(p_id, channel) => instance.connect(p_id, channel).await,
                    Close(p_id) => instance.close(p_id),
                }
            }
        });

        tx
    }

    pub async fn r#move(
        &mut self,
        player_id: PlayerId,
        game_id: GameId,
        from: Position,
        to: Position,
    ) {
        let game = match self.current_games.get_mut(&game_id) {
            Some(g) => g,
            None => return,
        };
        // println!("game found");

        let move_string_representation = game.move_piece(player_id, from, to);

        for id in game.players {
            let channel = self
                .current_players
                .get(&id)
                .expect("player with id should already have an active channel");

            let _ = channel
                .send(
                    serde_json::to_string(&json!({
                    "action": "move",
                    "game_id": game_id,
                    "data": game.get_moves(),
                    }))
                    .expect("Message to string serialization shouldn't fail"),
                )
                .await;
            let _ = channel
                .send(
                    serde_json::to_string(&json!({
                    "action": "move info",
                    "game_id": game_id,
                    "data": move_string_representation,
                    }))
                    .expect("Message to string serialization shouldn't fail"),
                )
                .await;
            println!("send to ws");
        }
        println!("end");
    }

    pub async fn connect(&mut self, player_id: PlayerId, channel: mpsc::Sender<WsMessageOutgoing>) {
        self.current_players.insert(player_id, channel.clone());

        for game in self.current_games.values() {
            if !game.players.contains(&player_id) {
                continue;
            }
            let init = serde_json::to_string(&json!( {
                "action": "init",
                "game_id": game.game_id,
                "data": {
                    "username": "test",
                },
            }))
            .expect("Message to string serialization shouldn't fail");
            let moves = serde_json::to_string(&json!( {
                "action": "move",
                "game_id": game.game_id,
                "data": game.get_moves(),
            }))
            .expect("Message to string serialization shouldn't fail");

            let _ = channel.send(init).await;
            let _ = channel.send(moves).await;
        }
    }

    pub async fn chat(&self, player_id: PlayerId, game_id: GameId, text: String) {
        println!("got here :)");
        let game = match self.current_games.get(&game_id) {
            Some(g) => g,
            None => return,
        };

        for opponent_id in game.players.iter().filter(|p_id| **p_id != player_id) {
            let n = json!({
             "action": "chat",
             "game_id": game_id,
             "data": text,
            });
            let channel = self.current_players.get(opponent_id).unwrap();
            let _ = channel
                .send(
                    serde_json::to_string(&n)
                        .expect("Message to string serialization shouldn't fail"),
                )
                .await;
            println!("sent");
        }
    }

    pub fn end(&self, player_id: PlayerId, game_id: GameId, reason: ChessEnd) {}
    pub async fn new_game(&mut self, player_id: PlayerId) {
        if let Some(waiting_id) = self.waiting_player {
            let players = [waiting_id, player_id];
            let game = ChessGame::new(players, [(); 2]);

            for player in players {
                let player_channel = self
                    .current_players
                    .get(&player)
                    .expect("when creating new game, game organizer should already have player's tx channel");

                #[derive(Debug, Serialize)]
                struct PlayerInfo {
                    username: String,
                }
                let player_info = sqlx::query_as!(
                    PlayerInfo,
                    "SELECT username FROM User
                    WHERE id = ?",
                    player as u64
                )
                .fetch_one(&self.db_pool)
                .await
                .expect("Player data query failed");

                let _ = player_channel
                    .send(
                        serde_json::to_string(&json!( {
                        "action": "init",
                        "game_id": game.game_id,
                        "data": &player_info,
                        }))
                        .expect("Message to string serialization shouldn't fail"),
                    )
                    .await;
                let _ = player_channel
                    .send(
                        serde_json::to_string(&json!( {
                        "action": "move",
                        "game_id": game.game_id,
                        "data": game.get_moves(),
                        }))
                        .expect("Message to string serialization shouldn't fail"),
                    )
                    .await;
                println!("Send to ws");
            }

            self.current_games.insert(game.game_id, game);
            self.waiting_player = None;
        } else {
            self.waiting_player = Some(player_id);
        }
    }

    pub fn close(&mut self, player_id: PlayerId) {
        self.current_players.remove(&player_id);
    }
}

#[derive(Debug, Clone)]
pub enum GameOrganizerRequest {
    Move(PlayerId, GameId, Position, Position),
    Chat(PlayerId, GameId, String),
    End(PlayerId, GameId, ChessEnd),
    NewGame(PlayerId),
    Connect(PlayerId, mpsc::Sender<WsMessageOutgoing>),
    Close(PlayerId),
}
