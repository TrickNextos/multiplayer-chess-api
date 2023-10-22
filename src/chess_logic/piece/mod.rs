mod direction;

use super::{Board, Player, Position};
pub use crate::chess_logic::piece::direction::*;

pub trait Piece
where
    Self: std::fmt::Debug,
{
    fn get_directions(&self) -> Vec<&'static dyn Direction>;
    fn get_directions_ids(&self) -> Vec<i32> {
        self.get_directions()
            .into_iter()
            .map(|piece_direction| piece_direction.direction_id())
            .collect()
    }
    fn get_piece_name(&self) -> String;
    fn get_moves(&self, board: &Board) -> Vec<Position> {
        let mut moves = Vec::new();
        for direction in self.get_directions() {
            for mut line in direction.get_all_moves(self.get_position(), self.get_player(), board) {
                moves.append(&mut line);
            }
        }
        moves
    }

    // getter & setter methods
    fn set_position(&mut self, position: Position);
    fn get_player(&self) -> Player;
    fn get_position(&self) -> Position;
}

macro_rules! piece_struct {
    ($piece_name: ident, $filename: literal, $($moving_direction: ident), *) => {
        #[derive(Debug, Clone)]
        pub struct $piece_name {
            position: Position,
            player: Player,
        }

        impl $piece_name {
            pub fn new(position: Position, player: Player) -> Self {
                Self { player, position }
            }
        }

        impl Piece for $piece_name {
            fn get_player(&self) -> Player {
                self.player
            }
            fn get_position(&self) -> Position {
                self.position
            }
            fn set_position(&mut self, position: Position) {
                self.position = position;
            }
            fn get_piece_name(&self) -> String {
                format!(
                    "{}{}",
                    match self.player {
                        Player::White => "w",
                        Player::Black => "b",
                    },
                    $filename,
                )
            }

            fn get_directions(&self) -> Vec<&'static dyn Direction> {
                vec![
                    $(
                        &$moving_direction {},
                    )*
                ]
            }
        }
    };
}

piece_struct!(Rook, "r", RookDirection);
piece_struct!(Bishop, "b", BishopDirection);
piece_struct!(Queen, "q", BishopDirection, RookDirection);
piece_struct!(King, "k", KingDirection);
piece_struct!(Knight, "n", KnightDirection);
piece_struct!(Pawn, "p", PawnEatingDirection, PawnMovingDirection);
