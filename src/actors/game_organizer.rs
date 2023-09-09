use std::collections::HashMap;

use actix::{Actor, Context, Handler, Message, Recipient};

use super::{
    game::{GameActor, StartStop},
    WsPlayer,
};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct AddNewPlayer(WsPlayer);

impl AddNewPlayer {
    pub fn new(player: WsPlayer) -> Self {
        Self(player)
    }
}

// TODO: implment system for checking which websockets have stopped and removed them from
// waiting_player and tell that to their game
#[derive(Debug, Default)]
pub struct GameOrganizer {
    current_games: HashMap<usize, Recipient<StartStop>>,
    waiting_player: Option<WsPlayer>,
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
    fn handle(&mut self, msg: AddNewPlayer, _ctx: &mut Self::Context) -> Self::Result {
        println!("player joined");
        if let Some(other_player) = self.waiting_player {
            let recipient = GameActor::new([msg.0, other_player]).start().recipient();
            self.current_games.insert(self.current_game_id, recipient);
            println!("New game created");
        } else {
            self.waiting_player = Some(msg.0);
        };
    }
}
