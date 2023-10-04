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
        vec![]
    }

    fn bishop_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        const DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
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
        const DIRECTIONS: [(i32, i32); 8] = [
            (0, 1),
            (0, -1),
            (1, 0),
            (-1, 0),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        let mut moves = Vec::new();
        for (x, y) in DIRECTIONS {
            if (0..8).contains(&(pos.x() + x)) && (0..8).contains(&(pos.x() + x)) {
                let mut new_pos = pos.clone();
                if let Ok(()) = new_pos.add(x, y) {
                    moves.push(vec![new_pos]);
                }
            }
        }
        moves
    }

    fn knight_moves(board: &Board, pos: Position, player: Player) -> Vec<Vec<Position>> {
        let mut moves = Vec::new();
        for i in [-2, 2] {
            for j in [-1, 1] {
                let mut pos_inner = pos.clone();
                if let Ok(()) = pos_inner.add(i, j) {
                    moves.push(vec![pos_inner]);
                }

                let mut pos_inner = pos.clone();
                if let Ok(()) = pos_inner.add(j, i) {
                    moves.push(vec![pos_inner]);
                }
            }
        }
        moves
    }
}
