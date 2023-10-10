use std::collections::HashMap;

use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};

use super::{
    game::{GameActor, OrganizerGameEnded, StartStop},
    ws_actions::{DataToWs, MessageFromWs},
    WsPlayer,
};

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct AddNewPlayer {
    id: WsPlayer,
    recipient: Recipient<DataToWs>,
}

impl AddNewPlayer {
    pub fn new(id: WsPlayer, recipient: Recipient<DataToWs>) -> Self {
        Self { id, recipient }
    }
}

// TODO: implment system for checking which websockets have stopped and removed them from
// waiting_player and tell that to their game
#[derive(Debug, Default)]
pub struct GameOrganizer {
    current_games: HashMap<usize, Recipient<MessageFromWs>>,
    waiting_player: Option<AddNewPlayer>,
    current_game_id: usize,
}

impl Actor for GameOrganizer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("test");
    }
}

impl Handler<AddNewPlayer> for GameOrganizer {
    type Result = ();
    fn handle(&mut self, msg: AddNewPlayer, ctx: &mut Self::Context) -> Self::Result {
        println!("player joined");
        if let Some(other_player) = self.waiting_player.clone() {
            let recipient = GameActor::new(
                [msg.id, other_player.id],
                [msg.recipient, other_player.recipient],
                ctx.address().recipient(),
            )
            .start()
            .recipient();
            self.current_games.insert(self.current_game_id, recipient);
            // TODO: get universal games id generator thingy, or maybe not
            self.current_game_id += 1;
            println!("New game created");
            self.waiting_player = None;
        } else {
            self.waiting_player = Some(msg);
        };
    }
}

impl Handler<OrganizerGameEnded> for GameOrganizer {
    type Result = ();
    fn handle(&mut self, msg: OrganizerGameEnded, ctx: &mut Self::Context) -> Self::Result {
        ()
    }
}
