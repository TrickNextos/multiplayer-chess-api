use crate::actors::ws_actions::PieceWithMoves;

use super::{board::Board, Position};

#[derive(Debug, Clone, Default)]
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
                    println!("piece: {}/{}", x, y);
                    moves.push(PieceWithMoves::new(
                        piece.get_filename(),
                        piece.position,
                        piece.get_moves(&self.board),
                    ))
                }
            }
        }
        moves
    }

    pub fn move_piece(&mut self, from: Position, to: Position) {
        println!("{:#?}", self.board);
        if let Some(piece) = self.board.remove(from) {
            self.board.set(to, piece);
        }
        println!("{:#?}", self.board);
    }
}
