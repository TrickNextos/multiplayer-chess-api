mod direction;
use direction::Direction;

use super::{Board, Player, Position};

#[derive(Clone, Debug)]
pub struct Piece {
    pub position: Position,
    directions: Vec<Direction>,
    pub player: Player,
    pub piece_name: &'static str,
}

impl Piece {
    fn create_with_direction(
        player: Player,
        position: Position,
        directions: Vec<Direction>,
        piece_name: &'static str,
    ) -> Self {
        Self {
            player,
            position,
            directions,
            piece_name,
        }
    }

    pub fn get_filename(&self) -> String {
        format!(
            "{}{}",
            match self.player {
                Player::White => "w",
                Player::Black => "b",
            },
            self.piece_name,
        )
    }

    pub fn rook(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Rook], "r")
    }

    pub fn pawn(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Pawn], "p")
    }

    pub fn bishop(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Bishop], "b")
    }

    pub fn king(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::King], "k")
    }

    pub fn queen(player: Player, position: Position) -> Self {
        Self::create_with_direction(
            player,
            position,
            vec![Direction::Bishop, Direction::Rook],
            "q",
        )
    }

    pub fn knight(player: Player, position: Position) -> Self {
        Self::create_with_direction(player, position, vec![Direction::Knight], "n")
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
