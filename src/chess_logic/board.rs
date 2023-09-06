use crate::chess_logic::Player;

use super::{piece::Piece, Position};

#[derive(Clone)]
pub struct Board([[Option<Piece>; 8]; 8]);

impl Board {
    pub fn get(&self, position: Position) -> Option<Piece> {
        self.0[position.y()][position.x()].clone()
    }
}

impl Default for Board {
    fn default() -> Self {
        const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0";
        Board::from_fen(STARTING_POSITION).expect("Default board fen should be correct")
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Self, &str> {
        // fen structure:
        // pieces_position current_player castle_rights en_passant_targets halfmove_clock fullmove_clock
        let mut board = Self::empty();
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

                    let position = Position(x as usize, y as usize);
                    x += 1;

                    let final_piece = match char.to_lowercase().to_string().as_str() {
                        "p" => Piece::new_temp(player, position),
                        "r" => Piece::new_temp(player, position),
                        "n" => Piece::new_temp(player, position),
                        "b" => Piece::new_temp(player, position),
                        "k" => Piece::new_temp(player, position),
                        "q" => Piece::new_temp(player, position),
                        _ => return Err("wrong piece name"),
                    };

                    board.0[y as usize][x as usize] = Some(final_piece);
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
