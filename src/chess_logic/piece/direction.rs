use crate::chess_logic::{board::Board, Player, Position};

#[derive(Clone, Copy)]
pub enum Direction {
    Pawn,
    Bishop,
    Rook,
    King,
    Knight,
}

impl Direction {
    fn get_moves(&self, board: &Board, pos: Position, player: Player) {
        match self {
            Direction::Pawn => Direction::pawn_moves(board, pos, player),
            Direction::Rook => Direction::rook_moves(board, pos, player),
            Direction::Bishop => Direction::bishop_moves(board, pos, player),
            Direction::King => Direction::king_moves(board, pos, player),
            Direction::Knight => Direction::knight_moves(board, pos, player),
        }
    }

    fn pawn_moves(board: &Board, pos: Position, player: Player) {
        todo!("moves not implemented yet");
    }

    fn bishop_moves(board: &Board, pos: Position, player: Player) {
        todo!("moves not implemented yet");
    }

    fn rook_moves(board: &Board, pos: Position, player: Player) {
        todo!("moves not implemented yet");
    }

    fn king_moves(board: &Board, pos: Position, player: Player) {
        todo!("moves not implemented yet");
    }

    fn knight_moves(board: &Board, pos: Position, player: Player) {
        todo!("moves not implemented yet");
    }
}
