use futures::future::join_all;
use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{
    api::game_ws::ChessEnd,
    chess_logic::{ChessGame, Position},
    sql::{self, PlayerData},
    GameId, PlayerId, WsMessageOutgoing,
};
use serde_json::{json, Value};
use sqlx::{MySql, Pool};

#[derive(Debug)]
pub struct GameOrganizer {
    current_games: HashMap<GameId, ChessGame>,
    waiting_player: Option<PlayerId>,
    current_players: HashMap<PlayerId, mpsc::Sender<String>>,

    pub pending_friend_requests: HashMap<u32, [PlayerId; 2]>,

    db_pool: Pool<MySql>,
}

impl GameOrganizer {
    pub fn new(db_pool: Pool<MySql>) -> mpsc::Sender<GameOrganizerRequest> {
        let mut instance = Self {
            db_pool,
            current_games: Default::default(),
            waiting_player: Default::default(),
            current_players: Default::default(),
            pending_friend_requests: Default::default(),
        };

        let (tx, mut rx) = mpsc::channel::<GameOrganizerRequest>(32);

        actix_rt::spawn(async move {
            while let Some(msg) = rx.recv().await {
                dbg!(msg.clone());
                use GameOrganizerRequest::*;
                match msg {
                    Move(p_id, g_id, from, to) => instance.r#move(p_id, g_id, from, to).await,
                    Chat(p_id, g_id, text) => instance.chat(p_id, g_id, text).await,
                    End(p_id, g_id, reason) => instance.end(p_id, g_id, reason).await,
                    NewGame(p_id) => instance.new_game(p_id).await,
                    Connect(p_id, channel) => instance.connect(p_id, channel).await,
                    Close(p_id) => instance.close(p_id),
                    FriendNew(r_id, p_id, f_id) => {
                        instance.new_friend_request(r_id, p_id, f_id).await
                    }
                    FriendAccept(r_id, p_id, f_id) => {
                        instance.finish_friend_request(r_id, p_id, f_id, true).await
                    }
                    FriendReject(r_id, p_id, f_id) => {
                        instance
                            .finish_friend_request(r_id, p_id, f_id, false)
                            .await
                    }
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
        let is_checkmate;
        {
            let game = match self.current_games.get_mut(&game_id) {
                Some(g) => g,
                None => return,
            };
            // println!("game found");

            if player_id != game.players[game.current_player_id] {
                return;
            }

            let (has_moves, move_string_representation) = match game.move_piece(from, to) {
                Ok(s) => s,
                Err(_) => return, // invalid move inserted
            };
            is_checkmate = !has_moves;

            game.current_move_data
                .push(move_string_representation.clone());

            for id in game.players {
                let channel = self
                    .current_players
                    .get(&id)
                    .expect("player with id should already have an active channel");

                // send legal moves only if you are the current player
                let move_data = {
                    if game.players[game.current_player_id] == id {
                        game.get_moves_as_json()
                    } else {
                        game.get_position_as_json()
                    }
                };

                let _ = channel
                    .send(
                        serde_json::to_string(&json!({
                        "action": "move",
                        "game_id": game_id,
                        "data": move_data,
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
            if is_checkmate {
                for id in game.players {
                    let channel = self
                        .current_players
                        .get(&id)
                        .expect("player with id should already have an active channel");

                    let _ = channel
                        .send(
                            serde_json::to_string(&json!({
                            "action": "end",
                            "game_id": game_id,
                            "data": {
                                "type": "checkmate",
                                "win": id == player_id,
                            },
                            }))
                            .expect("Message to string serialization shouldn't fail"),
                        )
                        .await;
                }
            }
        }

        // checkmate
        if is_checkmate {
            self.current_games.remove(&game_id);
        }

        println!("end");
    }

    pub async fn connect(&mut self, player_id: PlayerId, channel: mpsc::Sender<WsMessageOutgoing>) {
        self.current_players.insert(player_id, channel.clone());

        for game in self.current_games.values_mut() {
            if !game.players.contains(&player_id) {
                continue;
            }

            let opponent = match game
                .players
                .into_iter()
                .filter(|p_id| **p_id != player_id)
                .next()
            {
                Some(p_id) => crate::sql::get_player_data(&self.db_pool, *p_id as u64)
                    .await
                    .expect("Couldnt fetch user data from db"),
                None => PlayerData::singleplayer(),
            };

            let ask_draw = {
                if let Some(id) = game.current_draw_status {
                    Some(id != player_id)
                } else {
                    None
                }
            };

            let init = serde_json::to_string(&json!( {
                "action": "init",
                "game_id": game.game_id,
                "data": {
                    "opponent": opponent,
                    "chat": game.current_chat_data.iter().map(|(p_id, chat)| (*p_id == player_id, chat.clone()))
                            .collect::<Vec<(bool, String)>>(),
                    "moves": game.current_move_data.clone(),
                    "ask_draw": ask_draw,
                    "new_game": false,
                },
            }))
            .expect("Message to string serialization shouldn't fail");

            let move_data = {
                if player_id == game.players[game.current_player_id] {
                    game.get_moves_as_json()
                } else {
                    game.get_position_as_json()
                }
            };
            let moves = serde_json::to_string(&json!( {
                "action": "move",
                "game_id": game.game_id,
                "data": move_data,
            }))
            .expect("Message to string serialization shouldn't fail");

            let _ = channel.send(init).await;
            let _ = channel.send(moves).await;
        }
    }
    pub async fn chat(&mut self, player_id: PlayerId, game_id: GameId, text: String) {
        println!("got here :)");
        let game = match self.current_games.get_mut(&game_id) {
            Some(g) => g,
            None => return,
        };

        game.current_chat_data.push((player_id, text.clone()));

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

    pub async fn end(&mut self, player_id: PlayerId, game_id: GameId, reason: ChessEnd) {
        let game: &mut ChessGame = self
            .current_games
            .get_mut(&game_id)
            .expect("Game should already exist");
        match reason {
            ChessEnd::Resign => {
                for id in game.players {
                    self.send_to_player_ws(
                        id,
                        json!({
                        "action": "end",
                            "data": {
                                "type": "resign",
                                "win": id != player_id
                            }
                        }),
                    )
                    .await;
                }
                self.current_games.remove(&game_id);
            }
            ChessEnd::DrawConfirm => {
                if let Some(id) = game.current_draw_status {
                    if id == player_id {
                        // the other player must accept / deny the draw
                        return;
                    }
                }
                for id in game.players {
                    self.send_to_player_ws(
                        id,
                        json!({
                            "action": "end",
                                "data": {
                                    "type": "draw-confirm",
                            }
                        }),
                    )
                    .await;
                }
                self.current_games.remove(&game_id);
            }
            ChessEnd::DrawCancel => {
                if let Some(id) = game.current_draw_status {
                    if id == player_id {
                        // the other player must accept / deny the draw
                        return;
                    }
                }
                game.current_draw_status = None;

                for id in game.players {
                    self.send_to_player_ws(
                        id,
                        json!({
                            "action": "end",
                                "data": {
                                "type": "draw-cancel",
                            }
                        }),
                    )
                    .await;
                }
            }
            ChessEnd::DrawAsk => {
                game.current_draw_status = Some(player_id);
                for id in game.players {
                    self.send_to_player_ws(
                        id,
                        json!({
                            "action": "end",
                                "data": {
                                "type": "draw-ask",
                                "data": id != player_id
                            }
                        }),
                    )
                    .await;
                }
            }
        }
    }

    /// sends specified json value to a player
    async fn send_to_player_ws(&self, player_id: PlayerId, send_info: Value) {
        let _ = self
            .current_players
            .get(&player_id)
            .expect("Player should have websocket opened")
            .send(
                serde_json::to_string(&send_info)
                    .expect("Message to string serialization shouldn't fail"),
            )
            .await;
    }

    pub async fn new_game(&mut self, player_id: PlayerId) {
        if let Some(waiting_id) = self.waiting_player {
            let players = [waiting_id, player_id];
            let mut players_info: Vec<PlayerData> = join_all(vec![
                crate::sql::get_player_data(&self.db_pool, waiting_id as u64),
                crate::sql::get_player_data(&self.db_pool, player_id as u64),
            ])
            .await
            .into_iter()
            .map(|res| res.expect("Player data query failed"))
            .collect::<Vec<PlayerData>>()
            .into();

            let mut game = ChessGame::new(players);

            for (i, player) in players.iter().enumerate() {
                // check if this is singleplayer game
                if players[0] == players[1] {
                    if i == 1 {
                        continue;
                    }
                    players_info = vec![PlayerData::singleplayer(), PlayerData::singleplayer()];
                }

                let player_channel = self
                    .current_players
                    .get(&player)
                    .expect("when creating new game, game organizer should already have player's tx channel");

                let _ = player_channel
                    .send(
                        serde_json::to_string(&json!( {
                            "action": "init",
                            "game_id": game.game_id,
                            "data": {
                                "opponent": &players_info[1-i],
                                "moves": [],
                                "chat": "",
                                "new_game": true,
                            },
                        }))
                        .expect("Message to string serialization shouldn't fail"),
                    )
                    .await;

                // send legal moves only if you are the current player
                let move_data = {
                    if i == game.current_player_id {
                        game.get_moves_as_json()
                    } else {
                        game.get_position_as_json()
                    }
                };
                let _ = player_channel
                    .send(
                        serde_json::to_string(&json!( {
                        "action": "move",
                        "game_id": game.game_id,
                        "data": move_data,
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

    pub async fn new_friend_request(
        &mut self,
        request_id: u32,
        player_id: PlayerId,
        friend_id: PlayerId,
    ) {
        let player_data = sql::get_player_data(&self.db_pool, player_id as u64)
            .await
            .expect("User should be in db");
        println!("myb?");
        self.pending_friend_requests
            .insert(request_id, [player_id, friend_id]);
        match self.current_players.get(&friend_id) {
            Some(sender) => {
                let _ = sender
                    .send(
                        serde_json::to_string(&json!({"action": "friend request", "data": {
                            "request_id": request_id,
                            "user": player_data,
                        }}))
                        .expect("Json to string shouldn't fail"),
                    )
                    .await;
                println!("send to ws");
            }
            None => {}
        }
    }
    pub async fn finish_friend_request(
        &mut self,
        request_id: u32,
        player_id: PlayerId,
        friend_id: PlayerId,
        accept: bool,
    ) {
        println!("entered");
        match self.pending_friend_requests.get(&request_id) {
            Some(array) => {
                if !(array[0] == player_id && array[1] == friend_id) {
                    return;
                }
            }
            None => return,
        };
        println!("get throught checks");
        if accept {
            match sqlx::query!(
                "INSERT into Friends(friend1, friend2) values (?, ?)",
                player_id as u64,
                friend_id as u64
            )
            .execute(&self.db_pool)
            .await
            {
                Ok(_) => {
                    let _ = self.pending_friend_requests.remove(&request_id);
                }
                Err(_) => {}
            }
        } else {
            let _ = self.pending_friend_requests.remove(&request_id);
        }
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

    FriendNew(u32, PlayerId, PlayerId),
    FriendAccept(u32, PlayerId, PlayerId),
    FriendReject(u32, PlayerId, PlayerId),
}
