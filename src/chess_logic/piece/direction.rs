use crate::chess_logic::{board::Board, Player, Position};

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Pawn,
    Bishop,
    Rook,
    King,
    Knight,
}

impl Direction {
    pub fn get_moves(&self, board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        match self {
            Direction::Pawn => Direction::pawn_moves(board, pos, player),
            Direction::Rook => Direction::rook_moves(board, pos, player),
            Direction::Bishop => Direction::bishop_moves(board, pos, player),
            Direction::King => Direction::king_moves(board, pos, player),
            Direction::Knight => Direction::knight_moves(board, pos, player),
        }
    }

    fn pawn_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        todo!("moves not implemented yet");
    }

    fn bishop_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        todo!("moves not implemented yet");
    }

    fn rook_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        println!("Happens");
        DIRECTIONS
            .iter()
            .map(|(x_offset, y_offset)| {
                let mut dummy_pos = pos;

                let mut move_direction = Vec::new();
                while let Ok(()) = dummy_pos.add(*x_offset, *y_offset) {
                    move_direction.push(dummy_pos);
                }

                move_direction
            })
            .collect()
    }

    fn king_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        todo!("moves not implemented yet");
    }

    fn knight_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        todo!("moves not implemented yet");
    }
}
