mod board;
mod chess_game;
mod piece;

pub use board::Board;

#[derive(Clone, Copy, Debug)]
pub enum Player {
    White,
    Black,
}

#[derive(Clone, Copy, Debug)]
pub struct Position(i32, i32);

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        assert!((0..8).contains(&x));
        assert!((0..8).contains(&y));

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
        self.0 += x;
        self.1 += y;

        if (0..8).contains(&self.0) && (0..8).contains(&self.1) {
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
