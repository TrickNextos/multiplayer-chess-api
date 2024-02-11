use crate::chess_logic::{board::Board, Player, Position};

pub trait Direction {
    fn get_all_moves(&self, pos: Position, player: Player, board: &Board) -> Vec<Vec<Position>>;
    fn direction_id(&self) -> i32;
    fn extra_req(&self, _board: &Board, _pos: Position, _player: Player) -> bool {
        false
    }
}

// TODO: impl en passant and castling :)
pub struct RookDirection();
impl Direction for RookDirection {
    fn get_all_moves(&self, pos: Position, _player: Player, _board: &Board) -> Vec<Vec<Position>> {
        const DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];
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

    fn direction_id(&self) -> i32 {
        0
    }
}

pub struct BishopDirection();
impl Direction for BishopDirection {
    fn get_all_moves(&self, pos: Position, _player: Player, _board: &Board) -> Vec<Vec<Position>> {
        const DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
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

    fn direction_id(&self) -> i32 {
        1
    }
}

pub struct KingDirection();
impl Direction for KingDirection {
    fn get_all_moves(&self, pos: Position, _player: Player, _board: &Board) -> Vec<Vec<Position>> {
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

    fn direction_id(&self) -> i32 {
        2
    }
}

pub struct PawnEatingDirection();
impl Direction for PawnEatingDirection {
    fn get_all_moves(&self, pos: Position, player: Player, board: &Board) -> Vec<Vec<Position>> {
        let mut moves = Vec::new();
        let direction = match player {
            Player::White => -1,
            Player::Black => 1,
        };

        for i in [-1, 1] {
            let mut eating_pos = pos.clone();
            if eating_pos.add(i, direction).is_err() {
                continue;
            }

            if let Some(piece) = board.get(eating_pos) {
                if piece.get_player() != player {
                    moves.push(vec![eating_pos]);
                }
            }
        }

        moves
    }

    fn direction_id(&self) -> i32 {
        3
    }
}

pub struct PawnMovingDirection();
impl Direction for PawnMovingDirection {
    fn get_all_moves(&self, pos: Position, player: Player, board: &Board) -> Vec<Vec<Position>> {
        let mut moves = Vec::new();
        let direction = match player {
            Player::White => -1,
            Player::Black => 1,
        };

        let mut pos_copy = pos.clone();
        if pos_copy.add(0, direction).is_ok() && board.get(pos_copy).is_none() {
            moves.push(vec![pos_copy]);
        }

        if player == Player::White && pos.y() == 6 || player == Player::Black && pos.y() == 1 {
            let extra_position = Position(pos.x(), pos.y() + 2 * direction);
            if board.get(extra_position).is_none() && moves.len() >= 1 {
                moves[0].push(extra_position);
            }
        }

        moves
    }

    fn direction_id(&self) -> i32 {
        4
    }
}

pub struct KnightDirection();
impl Direction for KnightDirection {
    fn get_all_moves(&self, pos: Position, _player: Player, _board: &Board) -> Vec<Vec<Position>> {
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

    fn direction_id(&self) -> i32 {
        5
    }
}
