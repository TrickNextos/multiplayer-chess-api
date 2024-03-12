use super::direction::*;
use super::{Board, Player, Position};

pub trait Piece
where
    Self: std::fmt::Debug + Send,
{
    fn get_directions(&self) -> Vec<&'static dyn Direction>;
    fn get_directions_ids(&self) -> Vec<i32> {
        self.get_directions()
            .into_iter()
            .map(|piece_direction| piece_direction.direction_id())
            .collect()
    }
    fn get_piece_name(&self) -> String;
    fn get_moves(&self, board: &Board) -> Vec<(Position, i32)> {
        let mut moves = Vec::new();
        for direction in self.get_directions() {
            for line in direction.get_all_moves(self.get_position(), self.get_player(), board) {
                for piece_move in line {
                    if let Some(piece) = board.get(piece_move) {
                        if piece.get_player() != self.get_player() {
                            moves.push((piece_move, direction.direction_id()));
                        }
                        break;
                    }
                    moves.push((piece_move, direction.direction_id()));
                }
            }
        }
        moves
    }

    // getter & setter methods
    fn set_position(&mut self, position: Position);
    fn get_player(&self) -> Player;
    fn get_position(&self) -> Position;
    fn moved_yet(&self) -> bool;
    fn get_filename(&self) -> &'static str;
}

macro_rules! piece_struct {
    ($piece_name: ident, $filename: literal,  $($moving_direction: ident), *) => {
        #[derive(Debug, Clone)]
        pub struct $piece_name {
            position: Position,
            player: Player,
            moved_yet: bool,
            last_pos: Position,
        }

        impl $piece_name {
            pub fn new(position: Position, player: Player) -> Self {
                Self {
                    player,
                    position,
                    moved_yet: false,
                    last_pos: position
                }
            }
        }

        impl Piece for $piece_name {
            fn moved_yet(&self) -> bool {
                self.moved_yet
            }
            fn get_player(&self) -> Player {
                self.player
            }
            fn get_position(&self) -> Position {
                self.position
            }
            fn set_position(&mut self, position: Position) {
                self.last_pos = self.position;
                self.position = position;
                self.moved_yet = true;
            }
            fn get_filename(&self) -> &'static str {
                $filename
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
piece_struct!(King, "k", KingDirection, CastleDirection);
piece_struct!(Knight, "n", KnightDirection);
piece_struct!(
    Pawn,
    "p",
    PawnEatingDirection,
    PawnMovingDirection,
    EnPassantDirection
);
