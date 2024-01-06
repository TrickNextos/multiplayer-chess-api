use actix::{Actor, ActorContext, AsyncContext, Handler, Message, Recipient, StreamHandler};
use actix_web_actors::ws;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

use super::{
    game_organizer::CreateNewGame,
    ws_actions::{DataToWs, MessageFromWs, OuterMessageFromWs},
    WsPlayer,
};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub enum WsGameStartStop {
    Start(Recipient<MessageFromWs>),
    Stop,
}

pub struct ChessGameWs {
    player: WsPlayer,
    game_organizer: Arc<Recipient<CreateNewGame>>,
    game_actors: HashMap<u32, Recipient<MessageFromWs>>,
}

impl Actor for ChessGameWs {
    type Context = ws::WebsocketContext<Self>;
}

impl ChessGameWs {
    pub fn new(player_id: usize, game_organizer: Arc<Recipient<CreateNewGame>>) -> Self {
        let player = WsPlayer::new(player_id);
        Self {
            player,
            game_organizer,
            game_actors: HashMap::new(),
        }
    }

    fn handle_message(&mut self, msg: &str, rec: Recipient<DataToWs>) -> Result<(), String> {
        println!("Neki1 {}", msg);
        let message = serde_json::from_str::<OuterMessageFromWs>(msg)
            .map_err(|err| format!("Wrong data format {:?}", err))?;
        println!("Neki2");
        println!("Message: {:?}", message);
        println!("ChessGameWs game recepients: {:?}", self.game_actors);
        match message.game {
            Some(game_id) => match self.game_actors.get(&game_id) {
                Some(actor) => {
                    let inner_message =
                        MessageFromWs::new_message(self.player, message.data.as_str())?;
                    actor.do_send(inner_message);
                    println!("send message to actor");
                    Ok(())
                }
                None => Err("Game not started yet".to_owned()),
            },
            None => {
                println!("Send for createing a new game");
                self.game_organizer
                    .do_send(CreateNewGame::new(self.player, rec));
                Ok(())
            }
        }
    }
}

impl Handler<DataToWs> for ChessGameWs {
    type Result = Result<(), String>;
    fn handle(&mut self, msg: DataToWs, ctx: &mut Self::Context) -> Self::Result {
        println!("Data got to ws actor WOW");
        match msg {
            DataToWs::Init(id, player_data, game_actor) => {
                println!("Actor init somewhere");
                self.game_actors.insert(id, game_actor);
                println!("new game id: {}", id);
                ctx.text(
                    json!({"action": "init", "data": {
                    "id": id, "username": player_data.username,
                    }})
                    .to_string(),
                )
            }
            DataToWs::End(_reason) => ctx.stop(),
            DataToWs::Message(id, msg) => ctx.text(msg.serialize(id)),
        }
        Ok(())
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChessGameWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("Message in ws: {:?}", msg);
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Text(text)) => {
                if let Err(e) = self.handle_message(&text, ctx.address().recipient()) {
                    ctx.text(e);
                }
            }

            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            _ => (),
        }
    }

    // fn started(&mut self, ctx: &mut Self::Context) {
    //     let recipient: Recipient<DataToWs> = ctx.address().recipient();
    //     self.game_organizer
    //         .do_send(CreateNewGame::new(self.player, recipient));
    // }
}

// impl Handler<
