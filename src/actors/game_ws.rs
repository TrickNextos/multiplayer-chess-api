use actix::{Actor, ActorContext, AsyncContext, Handler, Message, Recipient, StreamHandler};
use actix_web_actors::ws;
use std::sync::Arc;

use super::{
    game_organizer::AddNewPlayer,
    ws_actions::{DataToWs, MessageFromWs, MessageToWs},
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
    game_organizer: Arc<Recipient<AddNewPlayer>>,
    game_actor: Option<Recipient<MessageFromWs>>,
}

impl Actor for ChessGameWs {
    type Context = ws::WebsocketContext<Self>;
}

impl ChessGameWs {
    pub fn new(player_id: usize, game_organizer: Arc<Recipient<AddNewPlayer>>) -> Self {
        let player = WsPlayer::new(player_id);
        Self {
            player,
            game_organizer,
            game_actor: None,
        }
    }

    fn handle_message(&mut self, msg: &str) -> Result<(), String> {
        let message = MessageFromWs::new_message(self.player, msg)?;
        println!("Message: {:?}", message);
        println!("ChessGameWs game recepients: {:?}", self.game_actor);
        match self.game_actor.clone() {
            Some(actor) => {
                actor.do_send(message);
                println!("send message to actor");
                Ok(())
            }
            None => Err("Game not started yet".to_owned()),
        }
    }
}

impl Handler<DataToWs> for ChessGameWs {
    type Result = Result<(), String>;
    fn handle(&mut self, msg: DataToWs, ctx: &mut Self::Context) -> Self::Result {
        println!("Data got to ws actor WOW");
        match msg {
            DataToWs::Init(game_actor) => self.game_actor = Some(game_actor),
            DataToWs::End(_reason) => ctx.stop(),
            DataToWs::Message(msg) => ctx.text(msg.serialize()),
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

    fn started(&mut self, ctx: &mut Self::Context) {
        let recipient: Recipient<DataToWs> = ctx.address().recipient();
        self.game_organizer
            .do_send(AddNewPlayer::new(self.player, recipient));
    }
}

// impl Handler<
