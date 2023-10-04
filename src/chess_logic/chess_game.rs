use crate::actors::ws_actions::PieceWithMoves;

use super::{board::Board, Position};

#[derive(Debug, Default)]
pub struct ChessGame {
    board: Board,
    // rules: Box<dyn ChessRule>
}

impl ChessGame {
    pub fn get_moves(&self) -> Vec<PieceWithMoves> {
        let mut moves = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.board.get(Position::new(x, y)) {
                    if y > 1 && y < 5 {
                        println!("piece: {}/{}", x, y);
                    }
                    moves.push(PieceWithMoves::new(
                        piece.get_piece_name(),
                        piece.get_position(),
                        piece.get_moves(),
                    ))
                }
            }
        }
        moves
    }

    pub fn move_piece(&mut self, from: Position, to: Position) {
        self.board.move_piece(from, to);
    }
}
