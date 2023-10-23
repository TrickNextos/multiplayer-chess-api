use std::collections::HashSet;

use crate::actors::ws_actions::PieceWithMoves;

use super::{
    board::Board,
    piece::{BishopDirection, Direction, KnightDirection, PawnEatingDirection, RookDirection},
    Player, Position,
};

#[derive(Debug)]
pub struct ChessGame {
    board: Board,
    king_positions: [Position; 2],
    current_player: Player,
    // rules: Box<dyn ChessRule>
}

impl Default for ChessGame {
    fn default() -> Self {
        Self {
            board: Board::default(),
            king_positions: [Position(4, 7), Position(4, 0)],
            current_player: Player::White,
        }
    }
}

impl ChessGame {
    pub fn get_moves(&self) -> Vec<PieceWithMoves> {
        let mut moves: [[Option<Vec<Position>>; 8]; 8] = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];

        const CHECKABLE_DIRECTIONS: [&'static dyn Direction; 4] = [
            &BishopDirection(),
            &RookDirection(),
            &PawnEatingDirection(),
            &KnightDirection(),
        ];

        for y in 0..8 {
            for x in 0..8 {
                if let Some(piece) = self.board.get(Position(x, y)) {
                    moves[y as usize][x as usize] = Some(piece.get_moves(&self.board));
                }
            }
        }

        let mut is_in_check = None;
        // all directions that can 'capture' the king
        for direction in CHECKABLE_DIRECTIONS {
            let all_moves = direction.get_all_moves(
                self.king_positions[match self.current_player {
                    Player::White => 0,
                    Player::Black => 1,
                }],
                self.current_player,
                &self.board,
            );

            for line in all_moves {
                // get moves for each direction
                let mut line_moves = HashSet::new();
                for position in line {
                    line_moves.insert(position);
                    if let Some(piece) = self.board.get(position) {
                        // if a piece can threaten king (is in his sightline e.g. diagonal)
                        // check which player's it is
                        if piece.get_player() != self.current_player
                            && piece // check if the piece can move in selected direction
                                .get_directions_ids()
                                .contains(&direction.direction_id())
                        {
                            match is_in_check {
                                None => is_in_check = Some(line_moves),
                                Some(mut vec) => {
                                    vec.extend(&mut line_moves.into_iter());
                                    is_in_check = Some(vec)
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        println!("is in check: {:?}", is_in_check);
        if let Some(legal_moves) = is_in_check {
            println!("I AM IN CHEEEECK");
            for y in 0..8 {
                for x in 0..8 {
                    if let Some(piece_moves) = moves[y as usize][x as usize].take() {
                        let mut piece_legal_moves = Vec::new();
                        for piece_move in piece_moves {
                            if legal_moves.contains(&piece_move) {
                                piece_legal_moves.push(piece_move);
                            }
                        }
                        moves[y as usize][x as usize] = Some(piece_legal_moves);
                    } else {
                        moves[y as usize][x as usize] = None;
                    }
                }
            }
        } else {
        }

        let mut final_moves = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                if let Some(legal_moves) = moves[y as usize][x as usize].take() {
                    let piece = self
                        .board
                        .get(Position::new(x, y))
                        .as_ref()
                        .expect("If a piece has moves it must exists");
                    if y > 1 && y < 5 {
                        println!("piece: {}/{}", x, y);
                    }
                    final_moves.push(PieceWithMoves::new(
                        piece.get_piece_name(),
                        piece.get_position(),
                        legal_moves,
                    ))
                }
            }
        }
        final_moves
    }

    pub fn move_piece(&mut self, from: Position, to: Position) {
        self.board.move_piece(from, to);
    }
}
