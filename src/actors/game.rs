use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};

use crate::{
    actors::ws_actions::{DataToWs, MessageFromWsType, MessageToWs},
    chess_logic::{ChessGame, Position},
};

use super::{
    ws_actions::{MessageFromWs, PlayerData},
    WsPlayer,
};

#[derive(Message)]
#[rtype(result = "()")]
pub enum StartStop {
    Start([(WsPlayer, WsPlayer); 2]),
    Stop,
}

#[derive(Debug, Message)]
#[rtype("result = ()")]
pub enum GameEnded {
    Win(WsPlayer),
    Draw,
}

#[derive(Debug)]
pub struct GameId(usize);

#[derive(Debug, Message)]
#[rtype("result = ()")]
pub struct OrganizerGameEnded {
    game_id: GameId,
    game_ended: GameEnded,
}

pub struct GameActor {
    id: u32,
    players: [WsPlayer; 2],
    game_ws_recipients: [Recipient<DataToWs>; 2],
    organizer_recipient: Recipient<OrganizerGameEnded>,
    player_info: [String; 2],

    chess_game: ChessGame,
}

impl GameActor {
    pub fn new(
        players: [WsPlayer; 2],
        game_ws_recipients: [Recipient<DataToWs>; 2],
        organizer_recipient: Recipient<OrganizerGameEnded>,
        player_info: [String; 2],
    ) -> Self {
        Self {
            id: rand::random(),
            players,
            game_ws_recipients,
            organizer_recipient,
            chess_game: ChessGame::default(),
            player_info,
        }
    }
}

impl Actor for GameActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.game_ws_recipients[0].do_send(DataToWs::Init(
            self.id,
            PlayerData {
                username: self.player_info[1].clone(),
            },
            ctx.address().recipient(),
        ));
        self.game_ws_recipients[1].do_send(DataToWs::Init(
            self.id,
            PlayerData {
                username: self.player_info[0].clone(),
            },
            ctx.address().recipient(),
        ));
        println!("a random game actor started func");

        let moves_for_ws = self.chess_game.get_moves();
        for (i, _player) in self.players.iter().enumerate() {
            self.game_ws_recipients[i].do_send(DataToWs::Message(
                self.id,
                MessageToWs::Moves(moves_for_ws.clone()),
            ));
        }
    }
}

impl Handler<MessageFromWs> for GameActor {
    type Result = Result<(), String>;
    fn handle(&mut self, msg: MessageFromWs, ctx: &mut Self::Context) -> Self::Result {
        println!("GameActor {:?}", msg);
        match msg.data {
            MessageFromWsType::Move(moving_pos) => {
                println!("move happened");
                self.chess_game.move_piece(moving_pos.from, moving_pos.to);

                let moves_for_ws = self.chess_game.get_moves();
                for (i, _player) in self.players.iter().enumerate() {
                    self.game_ws_recipients[i].do_send(DataToWs::Message(
                        self.id,
                        MessageToWs::Moves(moves_for_ws.clone()),
                    ));
                    self.game_ws_recipients[i].do_send(DataToWs::Message(
                        self.id,
                        MessageToWs::MoveInfo(format!("{} -> {}", moving_pos.from, moving_pos.to)),
                    ))
                }
            }
            MessageFromWsType::Premove(moving_pos) => {
                todo!("premoves not implemented yet")
            }
            MessageFromWsType::Chat(text) => {
                for (i, _player) in self.players.iter().enumerate() {
                    if msg.id == self.players[i] {
                        continue;
                    }
                    self.game_ws_recipients[i]
                        .do_send(DataToWs::Message(self.id, MessageToWs::Chat(text.clone())));
                }
            }
        }
        Ok(())
    }
}
