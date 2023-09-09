use actix::{Actor, Context, Handler, Message};

use super::{ws_actions::MessageFromWs, WsPlayer};

#[derive(Message)]
#[rtype(result = "()")]
pub enum StartStop {
    Start([(WsPlayer, WsPlayer); 2]),
    Stop,
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum GameEnded {
    Win(WsPlayer),
    Draw,
}

pub struct GameActor {
    players: [WsPlayer; 2],
}

impl GameActor {
    pub fn new(players: [WsPlayer; 2]) -> Self {
        Self { players }
    }
}

impl Actor for GameActor {
    type Context = Context<Self>;
}

// TODO: impl StartStop
impl Handler<StartStop> for GameActor {
    type Result = ();
    fn handle(&mut self, msg: StartStop, ctx: &mut Self::Context) -> Self::Result {}
}

impl Handler<MessageFromWs> for GameActor {
    type Result = Result<(), String>;
    fn handle(&mut self, msg: MessageFromWs, ctx: &mut Self::Context) -> Self::Result {
        println!("GameActor {:?}", msg);
        Ok(())
    }
}
