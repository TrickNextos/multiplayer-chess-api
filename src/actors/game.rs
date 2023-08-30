use actix::{Actor, Recipient, StreamHandler};
use actix_web_actors::ws;
use std::sync::Arc;

use super::ws_actions::MessageFromWs;

pub struct ChessGameWs {
    players: Vec<usize>,
    game_organizer: Arc<Recipient<MessageFromWs>>,
}

impl Actor for ChessGameWs {
    type Context = ws::WebsocketContext<Self>;
}

impl ChessGameWs {
    pub fn new(players: Vec<usize>, game_organizer: Arc<Recipient<MessageFromWs>>) -> Self {
        Self {
            players,
            game_organizer,
        }
    }

    fn handle_message(&mut self, msg: &str) -> Result<(), String> {
        let message = MessageFromWs::deserialize(msg)?;
        println!("Message: {:?}", message);
        self.game_organizer.do_send(message);
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
                if let Err(e) = self.handle_message(&text) {
                    ctx.text(e);
                }
            }

            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            _ => (),
        }
    }
}
