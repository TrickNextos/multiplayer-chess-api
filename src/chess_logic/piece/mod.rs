mod direction;
use direction::Direction;

use crate::chess_logic::piece::direction::{
    BishopDirection, KingDirection, KnightDirection, PawnDirection,
};

use self::direction::RookDirection;

use super::{Board, Player, Position};

pub trait Piece
where
    Self: std::fmt::Debug,
{
    fn get_directions(&self) -> Vec<&'static dyn Direction>;
    fn get_piece_name(&self) -> String;
    fn get_moves(&self) -> Vec<Position> {
        let mut moves = Vec::new();
        for direction in self.get_directions() {
            for mut line in direction.get_all_moves(self.get_position()) {
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

#[derive(Debug, Clone)]
pub struct Rook {
    position: Position,
    player: Player,
}

impl Rook {
    pub fn new(position: Position, player: Player) -> Self {
        Self { player, position }
    }
}

impl Piece for Rook {
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
            "r",
        )
    }

    fn get_directions(&self) -> Vec<&'static dyn Direction> {
        vec![&RookDirection {}]
    }
}

#[derive(Debug, Clone)]
pub struct Bishop {
    position: Position,
    player: Player,
}

impl Bishop {
    pub fn new(position: Position, player: Player) -> Self {
        Self { player, position }
    }
}

impl Piece for Bishop {
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
            "b",
        )
    }

    fn get_directions(&self) -> Vec<&'static dyn Direction> {
        vec![&BishopDirection {}]
    }
}

#[derive(Debug, Clone)]
pub struct Queen {
    position: Position,
    player: Player,
}

impl Queen {
    pub fn new(position: Position, player: Player) -> Self {
        Self { player, position }
    }
}

impl Piece for Queen {
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
            "q",
        )
    }

    fn get_directions(&self) -> Vec<&'static dyn Direction> {
        vec![&RookDirection {}, &BishopDirection {}]
    }
}

#[derive(Debug, Clone)]
pub struct King {
    position: Position,
    player: Player,
}

impl King {
    pub fn new(position: Position, player: Player) -> Self {
        Self { player, position }
    }
}

impl Piece for King {
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
            "k",
        )
    }

    fn get_directions(&self) -> Vec<&'static dyn Direction> {
        vec![&KingDirection {}]
    }
}

#[derive(Debug, Clone)]
pub struct Knight {
    position: Position,
    player: Player,
}

impl Knight {
    pub fn new(position: Position, player: Player) -> Self {
        Self { player, position }
    }
}

impl Piece for Knight {
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
            "n",
        )
    }

    fn get_directions(&self) -> Vec<&'static dyn Direction> {
        vec![&KnightDirection {}]
    }
}

#[derive(Debug, Clone)]
pub struct Pawn {
    position: Position,
    player: Player,
}

impl Pawn {
    pub fn new(position: Position, player: Player) -> Self {
        Self { player, position }
    }
}

impl Piece for Pawn {
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
            "p",
        )
    }

    fn get_directions(&self) -> Vec<&'static dyn Direction> {
        vec![&PawnDirection {}]
    }
}
