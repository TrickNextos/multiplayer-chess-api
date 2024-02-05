mod board;
mod chess_game;
pub use chess_game::ChessGame;
pub mod direction;
pub mod piece;

pub use board::Board;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    White,
    Black,
}

impl Player {
    fn player_index(&self) -> usize {
        match self {
            Player::White => 0,
            Player::Black => 1,
        }
    }

    fn change_player(&mut self) {
        *self = self.opponent();
    }

    fn opponent(&self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Position(i32, i32);

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        assert!(
            (0..8).contains(&x),
            "new Position with x outside of bounds: {}",
            &x
        );
        assert!(
            (0..8).contains(&y),
            "new Position with y outside of bounds: {}",
            &y
        );

        Self(x, y)
    }

    pub fn x(&self) -> i32 {
        self.0
    }
    pub fn y(&self) -> i32 {
        self.1
    }
    pub fn get(&self) -> [i32; 2] {
        [self.0, self.1]
    }

    pub fn add(&mut self, x: i32, y: i32) -> Result<(), ()> {
        if (0..8).contains(&(self.0 + x)) && (0..8).contains(&(self.1 + y)) {
            self.0 += x;
            self.1 += y;

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_unchecked(&mut self, x: i32, y: i32) {
        self.0 += x;
        self.1 += y;

        debug_assert!(
            (0..8).contains(&self.0),
            "func Position::add x out of range 0..8 -> {}",
            self.0
        );
        debug_assert!(
            (0..8).contains(&self.1),
            "func Position::add y out of range 0..8 -> {}",
            self.1
        );
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self.x() {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => unreachable!("There shouldn't be any position out of range: {:?}", self),
        };
        write!(f, "{}{}", letter, 7 - self.y())
    }
}
