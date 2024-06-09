use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc};

use crate::{
    api::game_ws::{ChessEnd, NewGameOptions, SingleplayerMultiplayer},
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
    pub pending_match_requests: HashMap<PlayerId, HashSet<PlayerId>>,

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
            pending_match_requests: Default::default(),
        };

        let (tx, mut rx) = mpsc::channel::<GameOrganizerRequest>(32);

        actix_rt::spawn(async move {
            while let Some(msg) = rx.recv().await {
                dbg!(msg.clone());
                use GameOrganizerRequest::*;
                match msg {
                    Move(p_id, g_id, from, to) => {
                        instance.r#move(p_id, g_id, from, to).await;
                    }
                    Chat(p_id, g_id, text) => instance.chat(p_id, g_id, text).await,
                    End(p_id, g_id, reason) => {
                        instance.end(p_id, g_id, reason).await;
                    }
                    NewGame(p_id, options) => instance.new_game(p_id, options).await,
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
    ) -> Option<()> {
        let is_checkmate;
        {
            let game = self.current_games.get_mut(&game_id)?;

            if player_id != game.players[game.current_player_id] {
                return None;
            }

            let (has_moves, move_string_representation) = match game.move_piece(from, to) {
                Ok(s) => s,
                Err(_) => return None, // invalid move inserted
            };
            is_checkmate = !has_moves;

            game.current_move_data
                .push(move_string_representation.clone());

            for id in game.players {
                let channel = self.current_players.get(&id)?;

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
                    let channel = self.current_players.get(&id)?;

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
            println!("CHECKMATE");
            self.end_game(game_id, "win", player_id).await;
        }
        println!("end");
        Some(())
    }

    async fn init_chess_game(
        player_id: PlayerId,
        game: &mut ChessGame,
        channel: &mpsc::Sender<String>,
    ) {
        println!("{:?}", game.players_info);
        let opponent = match game
            .players_info
            .iter()
            .filter(|p| p.id as usize != player_id)
            .next()
            // if the players have the same id, then its singleplayer
        {
            Some(p) => p.clone(),
            None => PlayerData::singleplayer(player_id),
        };
        dbg!(opponent.clone());
        let black_or_white = {
            if game.players_info[0].id as usize == player_id {
                "white"
            } else {
                "black"
            }
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
                    "playing": black_or_white,
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

    pub async fn connect(&mut self, player_id: PlayerId, channel: mpsc::Sender<WsMessageOutgoing>) {
        self.current_players.insert(player_id, channel.clone());

        let channel = self
            .current_players
            .get(&player_id)
            .expect("Player should have a ws openened");

        for game in self.current_games.values_mut() {
            if !game.players.contains(&player_id) {
                continue;
            }

            Self::init_chess_game(player_id, game, channel).await;
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

    pub async fn end(
        &mut self,
        player_id: PlayerId,
        game_id: GameId,
        reason: ChessEnd,
    ) -> Option<()> {
        let win;
        {
            let game: &mut ChessGame = self.current_games.get_mut(&game_id)?;
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
                    win = "lose";
                }
                ChessEnd::DrawConfirm => {
                    if let Some(id) = game.current_draw_status {
                        if id == player_id {
                            // the other player must accept / deny the draw
                            return Some(());
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
                    win = "draw";
                }
                ChessEnd::DrawCancel => {
                    if let Some(id) = game.current_draw_status {
                        if id == player_id {
                            // the other player must accept / deny the draw
                            return Some(());
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
                    return Some(());
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
                    return Some(());
                }
            }
        }
        self.end_game(game_id, win, player_id).await
    }

    async fn end_game(&mut self, game_id: GameId, win: &str, player_id: PlayerId) -> Option<()> {
        let uuid = uuid::Uuid::new_v4();
        let game: &mut ChessGame = self.current_games.get_mut(&game_id)?;
        let _ = sqlx::query!(
            "Insert into Games(white, black, game_file_uuid, num_of_moves, win, singleplayer)
            values (?, ?, ?, ?, ?, ?)",
            game.players[0] as u64,
            game.players[1] as u64,
            uuid.to_string(),
            game.current_move_data.len() as u16,
            {
                match win {
                    "draw" => "draw",
                    "lose" => {
                        if game.players[0] != player_id {
                            "white"
                        } else {
                            "black"
                        }
                    }
                    "win" => {
                        if game.players[0] == player_id {
                            "white"
                        } else {
                            "black"
                        }
                    }
                    _ => unreachable!("Status should only be win, lose or draw"),
                }
            },
            game.players[0] == game.players[1],
        )
        .execute(&self.db_pool)
        .await;
        let _ = save_game(game.current_move_data.clone(), uuid).await;
        self.current_games.remove(&game_id);
        Some(())
    }

    /// sends specified json value to a player
    async fn send_to_player_ws(&self, player_id: PlayerId, send_info: Value) -> Option<()> {
        let _ = self
            .current_players
            .get(&player_id)?
            .send(
                serde_json::to_string(&send_info)
                    .expect("Message to string serialization shouldn't fail"),
            )
            .await;
        Some(())
    }

    pub async fn new_game(&mut self, player_id: PlayerId, options: NewGameOptions) {
        dbg!(options);
        match options.game_type {
            SingleplayerMultiplayer::Singleplayer => {
                println!("happens");
                let mut game = ChessGame::new(vec![
                    PlayerData::singleplayer(player_id),
                    PlayerData::singleplayer(player_id),
                ]);

                let player_channel = self
                    .current_players
                    .get(&player_id)
                    .expect("when creating new game, game organizer should already have player's tx channel");

                Self::init_chess_game(player_id, &mut game, player_channel).await;

                self.current_games.insert(game.game_id, game);
            }
            SingleplayerMultiplayer::Multiplayer => {
                let op_id;
                match options.opponent {
                    Some(opponent_id) => match self.pending_match_requests.get_mut(&opponent_id) {
                        // Opponent already asked to play
                        Some(opponent_match_req) if opponent_match_req.contains(&player_id) => {
                            opponent_match_req.remove(&player_id);
                            op_id = opponent_id;
                        }
                        // Opponent did't ask to play yet
                        _ => {
                            match self.pending_match_requests.get_mut(&player_id) {
                                // current player already has some pending friend requests
                                Some(n) => {
                                    n.insert(opponent_id);
                                }
                                // player has no friend requests
                                None => {
                                    let mut hs = HashSet::new();
                                    hs.insert(opponent_id);
                                    self.pending_match_requests.insert(player_id, hs);
                                }
                            }
                            // we have to wait footherr the  player to confirm, so return

                            // send friend request to opponents inbox
                            let opponent_data =
                                sql::get_player_data(&self.db_pool, opponent_id as u64)
                                    .await
                                    .expect("This player should exist");

                            match self.current_players.get(&opponent_id) {
                                Some(sender) => {
                                    let _ = sender
                                        .send(
                                            serde_json::to_string(&json!({"action": "request", "data": {
                                                "request_id": 0,
                                                "request_type": "game",
                                                "user": opponent_data,
                                                "opponent": player_id,
                                                "text": format!("Game request: <b>{}</b>", opponent_data.username),
                                            }}))
                                            .expect("Json to string shouldn't fail"),
                                        )
                                        .await;
                                    println!("send to ws");
                                }
                                None => {}
                            }
                            return;
                        }
                    },
                    None => {
                        if let Some(waiting_id) = self.waiting_player {
                            self.waiting_player = None;
                            op_id = waiting_id;
                        } else {
                            self.waiting_player = Some(player_id);
                            // you are the first player in queue
                            return;
                        }
                    }
                }

                let players = [op_id, player_id];
                let players_info: Vec<PlayerData> = join_all(vec![
                    crate::sql::get_player_data(&self.db_pool, op_id as u64),
                    crate::sql::get_player_data(&self.db_pool, player_id as u64),
                ])
                .await
                .into_iter()
                .map(|res| res.expect("Player data query failed"))
                .collect::<Vec<PlayerData>>()
                .into();

                let mut game = ChessGame::new(players_info);

                for player in players {
                    let player_channel = self
                            .current_players
                            .get(&player)
                            .expect("when creating new game, game organizer should already have player's tx channel");

                    Self::init_chess_game(player, &mut game, player_channel).await;
                }

                self.current_games.insert(game.game_id, game);
                self.waiting_player = None;
            }
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
                        serde_json::to_string(&json!({"action": "request", "data": {
                            "request_id": request_id,
                            "request_type": "friend",
                            "user": player_data,
                            "text": format!("Friend request: <b>{}</b>", player_data.username),
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

async fn save_game(moves: Vec<String>, uuid: uuid::Uuid) -> std::io::Result<()> {
    let mut f = File::create(std::path::Path::new(&format!("../games/{}.pgn", uuid))).await?;
    let mut text = String::new();
    moves.into_iter().enumerate().for_each(|(i, mv)| {
        println!("{}", mv);
        if i % 2 == 0 {
            text.push_str(&format!("{}. ", (i / 2) + 1));
        }
        text.push_str(&format!("{mv} "));
    });
    f.write_all(
        text.chars()
            .filter(|b| *b != '\0')
            .collect::<String>()
            .as_bytes(),
    )
    .await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub enum GameOrganizerRequest {
    Move(PlayerId, GameId, Position, Position),
    Chat(PlayerId, GameId, String),
    End(PlayerId, GameId, ChessEnd),
    NewGame(PlayerId, NewGameOptions),
    Connect(PlayerId, mpsc::Sender<WsMessageOutgoing>),
    Close(PlayerId),

    FriendNew(u32, PlayerId, PlayerId),
    FriendAccept(u32, PlayerId, PlayerId),
    FriendReject(u32, PlayerId, PlayerId),
}
