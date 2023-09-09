pub mod game;
pub mod game_organizer;
pub mod game_ws;
pub mod ws_actions;

#[derive(Debug, Clone, Copy)]
pub struct WsPlayer(usize);

impl WsPlayer {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn get(&self) -> usize {
        self.0
    }
}
