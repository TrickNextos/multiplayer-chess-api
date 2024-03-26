use std::collections::HashMap;

use crate::chess_logic::{board::Board, Player, Position};

use super::{chess_game::check_if_valid_king_pos, ChessGame, PositionWithDirection};

pub fn get_direction_from_id(id: i32) -> Box<dyn Direction + 'static> {
    match id {
        0 => Box::new(RookDirection {}),
        1 => Box::new(BishopDirection {}),
        2 => Box::new(KingDirection {}),
        3 => Box::new(PawnEatingDirection {}),
        4 => Box::new(PawnMovingDirection {}),
        6 => Box::new(EnPassantDirection {}),
        5 => Box::new(KnightDirection {}),
        7 => Box::new(CastleDirection {}),
        _ => unreachable!("There should only be 8 directions"),
    }
}

pub trait Direction {
    fn get_all_moves(&self, pos: Position, player: Player, board: &Board) -> Vec<Vec<Position>>;
    fn direction_id(&self) -> i32;
    fn extra_req(
        &self,
    ) -> Box<
        dyn Fn(
            &ChessGame,
            PositionWithDirection,
            PositionWithDirection,
            Player,
            &Board,
            &[[Option<Vec<PositionWithDirection>>; 8]; 8],
            &HashMap<Position, Vec<PositionWithDirection>>,
            &Vec<Position>,
        ) -> Result<bool, anyhow::Error>,
    > {
        Box::new(
            |_game: &ChessGame,
             _pos: PositionWithDirection,
             _new_pos: PositionWithDirection,
             _player: Player,
             _board: &Board,
             _moves: &[[Option<Vec<PositionWithDirection>>; 8]; 8],
             _pinned: &HashMap<Position, Vec<PositionWithDirection>>,
             _pinned_twice: &Vec<Position>| Ok(true),
        )
    }
    fn side_effect(
        &self,
        _pos: Position,
        _new_pos: Position,
        _player: Player,
        _chess_game: &mut ChessGame,
    ) {
    }
}

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
    fn side_effect(
        &self,
        pos: Position,
        new_pos: Position,
        player: Player,
        chess_game: &mut ChessGame,
    ) {
        if (pos.y() - new_pos.y()).abs() == 2 {
            chess_game.can_enpassant[player.opponent().player_index()] = Some(new_pos);
            println!("ENNAPPASDASIDBAW TIME");
        }
    }

    fn direction_id(&self) -> i32 {
        4
    }
}

pub struct EnPassantDirection();
impl Direction for EnPassantDirection {
    fn get_all_moves(&self, pos: Position, player: Player, board: &Board) -> Vec<Vec<Position>> {
        let mut moves = Vec::new();
        let direction = match player {
            Player::White => -1,
            Player::Black => 1,
        };

        for i in [-1, 1].iter() {
            let mut pos_copy = pos.clone();
            let mut pos_copy2 = pos.clone();
            if pos_copy.add(*i, direction).is_ok() && pos_copy2.add(*i, 0).is_ok() {
                if let Some(piece) = board.get(pos_copy2) {
                    if piece.get_filename() == "p" && piece.get_player() != player {
                        moves.push(vec![pos_copy]);
                    }
                }
            }
        }

        moves
    }

    fn extra_req(
        &self,
    ) -> Box<
        dyn Fn(
            &ChessGame,
            PositionWithDirection,
            PositionWithDirection,
            Player,
            &Board,
            &[[Option<Vec<PositionWithDirection>>; 8]; 8],
            &HashMap<Position, Vec<PositionWithDirection>>,
            &Vec<Position>,
        ) -> Result<bool, anyhow::Error>,
    > {
        Box::new(
            |game: &ChessGame,
             pos: PositionWithDirection,
             new_pos: PositionWithDirection,
             player: Player,
             _board: &Board,
             _moves: &[[Option<Vec<PositionWithDirection>>; 8]; 8],
             pinned: &HashMap<Position, Vec<PositionWithDirection>>,
             cant_enpassant: &Vec<Position>| {
                let direction = match player {
                    Player::White => -1,
                    Player::Black => 1,
                };

                let mut piece_pos = new_pos.0;
                piece_pos.add(0, -direction)?;
                println!("Enpassant player_index {}", player.player_index());
                println!("{:?}", game.can_enpassant[player.player_index()]);
                println!("to: {:?}, piece_pos: {:?}", new_pos, piece_pos);
                Ok(game.can_enpassant[player.player_index()]
                    .is_some_and(|opponent_pawn_pos| opponent_pawn_pos == piece_pos)
                    && pinned.get(&piece_pos).is_none()
                    && !cant_enpassant.contains(&piece_pos)
                    && !cant_enpassant.contains(&pos.0))
            },
        )
    }

    fn side_effect(
        &self,
        _pos: Position,
        mut new_pos: Position,
        player: Player,
        chess_game: &mut ChessGame,
    ) {
        let direction = match player {
            Player::White => -1,
            Player::Black => 1,
        };
        new_pos
            .add(0, -direction)
            .expect("When removing enpassant-ed pawn, pawn shouldn't be off board");
        chess_game.board.0[new_pos.y() as usize][new_pos.x() as usize] = None;
    }

    fn direction_id(&self) -> i32 {
        6
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

pub struct CastleDirection();
impl Direction for CastleDirection {
    fn get_all_moves(&self, pos: Position, player: Player, board: &Board) -> Vec<Vec<Position>> {
        [-1, 1]
            .into_iter()
            .filter_map(|direction| {
                if !(0 <= pos.x() + 2 * direction && pos.x() + 2 * direction < 8) {
                    return None;
                }
                if !board.get(pos).is_some_and(|king| !king.moved_yet()) {
                    return None;
                }
                let mut king_move = pos.clone();
                let _ = king_move.add(*direction, 0);
                while king_move.x() > 0 && king_move.x() < 7 {
                    if board.get(king_move).is_some() {
                        println!("piece in the way {}", king_move);
                        return None;
                    }
                    if !check_if_valid_king_pos(board, player, king_move, pos) {
                        println!("not valid king move {}", king_move);
                        return None;
                    }
                    let _ = king_move.add(*direction, 0);
                }
                match board.get(king_move) {
                    None => None,
                    Some(piece) => {
                        if piece
                            .get_directions_ids()
                            .contains(&RookDirection {}.direction_id())
                            && !piece.moved_yet()
                        {
                            println!("SUCCESS");
                            Some(vec![Position::new(pos.x() + 2 * direction, pos.y())])
                        } else {
                            println!("rook moved");
                            None
                        }
                    }
                }
            })
            .collect::<Vec<Vec<Position>>>()
    }

    fn side_effect(
        &self,
        pos: Position,
        new_pos: Position,
        _player: Player,
        chess_game: &mut ChessGame,
    ) {
        let direction = {
            if pos.x() - new_pos.x() > 0 {
                -1
            } else {
                1
            }
        };
        let old_rook_position = Position::new(
            {
                match direction {
                    1 => 7,
                    -1 => 0,
                    _ => unreachable!(""),
                }
            },
            pos.y(),
        );
        let new_rook_position = Position::new((pos.x() + new_pos.x()) / 2, pos.y());
        println!(
            "ROOOK SIDE EFFECT GO BRRR {} -> {}",
            old_rook_position, new_rook_position
        );

        // chess_game.board.0[new_rook_position.y() as usize][new_rook_position.x() as usize] =
        //     chess_game.board.0[old_rook_position.y() as usize][old_rook_position.x() as usize]
        //         .take();
        let rook = chess_game.board.0[old_rook_position.y() as usize]
            [old_rook_position.x() as usize]
            .take();

        if let Some(mut rook) = rook {
            rook.set_position(new_rook_position);
            chess_game.board.0[new_rook_position.y() as usize][new_rook_position.x() as usize] =
                Some(rook);
        }
    }

    fn direction_id(&self) -> i32 {
        7
    }
}
