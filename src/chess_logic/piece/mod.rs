mod direction;
use direction::Direction;

use super::{Board, Player, Position};

#[derive(Clone)]
pub struct Piece {
    position: Position,
    directions: Vec<Direction>,
    player: Player,
}

impl Piece {
    pub fn new_temp(player: Player, position: Position) -> Self {
        Piece {
            player,
            position,
            directions: vec![],
        }
    }
}

// pub trait Piece {
//     fn get_player(&self) -> usize;
//     fn get_directions(&self) -> Vec<Direction>;
//     fn get_moves(&self, board: &Board);
//     fn move_piece(&self);
// }
