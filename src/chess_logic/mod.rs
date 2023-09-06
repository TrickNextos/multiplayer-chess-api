mod board;
mod chess_game;
mod piece;

pub use board::Board;

#[derive(Clone, Copy)]
pub enum Player {
    White,
    Black,
}

#[derive(Clone, Copy)]
pub struct Position(usize, usize);

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        assert!((0..8).contains(&x));
        assert!((0..8).contains(&y));

        Self(x, y)
    }

    pub fn x(&self) -> usize {
        self.0
    }
    pub fn y(&self) -> usize {
        self.1
    }
    pub fn get(&self) -> [usize; 2] {
        [self.0, self.1]
    }
}
