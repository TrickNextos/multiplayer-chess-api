mod direction;
use direction::Direction;

use super::{Board, Player, Position};

#[derive(Clone, Debug)]
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

    fn create_with_direction(
        player: Player,
        position: Position,
        directions: Vec<Direction>,
    ) -> Self {
        Self {
            player,
            position,
            directions,
        }
    }

    pub fn rook(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Rook])
    }

    pub fn pawn(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Pawn])
    }

    pub fn bishop(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Bishop])
    }

    pub fn king(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::King])
    }

    pub fn queen(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Bishop, Direction::Rook])
    }

    pub fn knight(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Knight])
    }

    pub fn get_moves(&self, board: &Board) -> Vec<Position> {
        self.directions
            .iter()
            .map(|direction| direction.get_moves(board, self.position, self.player))
            .flatten()
            .flatten()
            .collect()
    }
}
