use crate::chess_logic::{
    piece::{Bishop, King, Knight, Pawn, Queen, Rook},
    Player,
};

use super::{piece::Piece, Position};

#[derive(Debug)]
pub struct Board([[Option<Box<dyn Piece>>; 8]; 8]);

impl Board {
    pub fn get(&self, position: Position) -> &Option<Box<dyn Piece>> {
        &self.0[position.y() as usize][position.x() as usize]
    }

    pub fn move_piece(&mut self, from: Position, to: Position) {
        dbg!(from);
        dbg!(to);

        if let Some(mut piece) = self.0[from.y() as usize][from.x() as usize].take() {
            piece.set_position(to);
            self.0[to.y() as usize][to.x() as usize] = Some(piece);
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0";
        Board::from_fen(STARTING_POSITION).expect("Default board fen should be correct")
    }
}

impl Board {
    // TODO: finish full fen implementation
    pub fn from_fen(fen: &str) -> Result<Self, &str> {
        // fen structure:
        // pieces_position current_player castle_rights en_passant_targets halfmove_clock fullmove_clock
        let mut board = Self::empty();
        println!("fen input: {}", fen);
        let pieces: Vec<&str> = fen.trim().split(' ').collect();
        if pieces.len() != 6 {
            return Err("Wrong amount of fen groups");
        }

        println!("For now, fen supports only piece position and current player");

        pieces[0]
            .split('/')
            .enumerate()
            .map(|(y, row)| {
                let mut x = 0;
                for char in row.chars() {
                    if let Some(digit) = char.to_digit(10) {
                        x += digit;
                        continue;
                    }
                    let player = match char.is_lowercase() {
                        true => Player::Black,
                        false => Player::White,
                    };

                    let position = Position(x as i32, y as i32);

                    let final_piece: Box<dyn Piece> = match char.to_lowercase().to_string().as_str()
                    {
                        "p" => Box::new(Pawn::new(position, player)),
                        "r" => Box::new(Rook::new(position, player)),
                        "n" => Box::new(Knight::new(position, player)),
                        "b" => Box::new(Bishop::new(position, player)),
                        "k" => Box::new(King::new(position, player)),
                        "q" => Box::new(Queen::new(position, player)),
                        _ => return Err("wrong piece name"),
                    };

                    board.0[y as usize][x as usize] = Some(final_piece);
                    x += 1;
                }
                Ok(())
            })
            .collect::<Result<(), &str>>()?;

        Ok(board)
    }

    pub fn empty() -> Self {
        Board([
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ])
    }
}
