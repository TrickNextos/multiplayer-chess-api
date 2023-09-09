use actix::{Actor, ActorContext, Handler, Message, Recipient, StreamHandler};
use actix_web_actors::ws;
use std::sync::Arc;

use super::{game_organizer::AddNewPlayer, ws_actions::MessageFromWs, WsPlayer};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub enum WsGameStartStop {
    Start(Recipient<MessageFromWs>),
    Stop,
}

pub struct ChessGameWs {
    player: WsPlayer,
    game_organizer: Arc<Recipient<AddNewPlayer>>,
    game_actor: Option<Recipient<MessageFromWs>>,
}

impl Actor for ChessGameWs {
    type Context = ws::WebsocketContext<Self>;
}

impl ChessGameWs {
    pub fn new(player_id: usize, game_organizer: Arc<Recipient<AddNewPlayer>>) -> Self {
        let player = WsPlayer::new(player_id);
        game_organizer.do_send(AddNewPlayer::new(player));
        Self {
            player,
            game_organizer,
            game_actor: None,
        }
    }

    fn handle_message(&mut self, msg: &str) -> Result<(), String> {
        let message = MessageFromWs::new_message(self.player, msg)?;
        println!("Message: {:?}", message);
        match self.game_actor.clone() {
            Some(actor) => {
                actor.do_send(message);
                Ok(())
            }
            None => Err("Game not started yet".to_owned()),
        }
    }
}

impl Handler<WsGameStartStop> for ChessGameWs {
    type Result = ();
    fn handle(&mut self, msg: WsGameStartStop, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            WsGameStartStop::Start(actor) => self.game_actor = Some(actor),
            WsGameStartStop::Stop => ctx.stop(),
        }
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
                    ctx.text(text);
                }
            }

            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            _ => (),
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {}
}

// impl Handler<
