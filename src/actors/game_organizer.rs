use std::{collections::HashMap, default};

use crate::sql;
use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};
use sqlx::{MySql, Pool};

use super::{
    game::{GameActor, OrganizerGameEnded, StartStop},
    ws_actions::{DataToWs, MessageFromWs},
    WsPlayer,
};

#[derive(Debug, Message, Clone)]
#[rtype(result = "()")]
pub struct CreateNewGame {
    id: WsPlayer,
    recipient: Recipient<DataToWs>,
    play_with: Option<WsPlayer>,
}

impl CreateNewGame {
    pub fn new(id: WsPlayer, recipient: Recipient<DataToWs>) -> Self {
        Self {
            id,
            recipient,
            play_with: None,
        }
    }
}

// TODO: implment system for checking which websockets have stopped and removed them from
// waiting_player and tell that to their game
#[derive(Debug)]
pub struct GameOrganizer {
    current_games: HashMap<usize, Recipient<MessageFromWs>>,
    waiting_player: Option<CreateNewGame>,
    current_game_id: usize,

    db_pool: Pool<MySql>,
}

impl GameOrganizer {
    pub fn new(db_pool: Pool<MySql>) -> Self {
        Self {
            db_pool,
            current_games: Default::default(),
            waiting_player: Default::default(),
            current_game_id: Default::default(),
        }
    }
}

impl Actor for GameOrganizer {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("test");
    }
}

impl Handler<CreateNewGame> for GameOrganizer {
    type Result = ();
    fn handle(&mut self, msg: CreateNewGame, ctx: &mut Self::Context) -> Self::Result {
        println!("player joined");
        if let Some(other_player) = self.waiting_player.clone() {
            let recipient = GameActor::new(
                [msg.id, other_player.id],
                [msg.recipient, other_player.recipient],
                ctx.address().recipient(),
                // TODO: use async in here somehow
                // URGENT: Refactor to actorless
                // [
                //     sql::get_player_data(&self.db_pool, msg.id.0 as u64),
                //     sql::get_player_data(&self.db_pool, other_player.id.0 as u64),
                // ],
                ["User1".to_owned(), "User2".to_owned()],
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
