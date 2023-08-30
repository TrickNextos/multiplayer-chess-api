use actix::{Actor, Context, Handler};

use super::ws_actions::MessageFromWs;

pub struct GameOrganizer;

impl Actor for GameOrganizer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("test");
    }
}

impl Handler<MessageFromWs> for GameOrganizer {
    type Result = Result<(), String>;
    fn handle(&mut self, msg: MessageFromWs, ctx: &mut Self::Context) -> Self::Result {
        println!("GameOrganizer{:?}", msg);
        Ok(())
    }
}
