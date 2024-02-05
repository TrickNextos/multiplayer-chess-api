use serde_json::{json, Value};
use std::collections::HashMap;

use super::{
    board::Board,
    direction::{BishopDirection, Direction, KnightDirection, PawnEatingDirection, RookDirection},
    Player, Position,
};
use crate::{GameId, PlayerId};

#[derive(Debug)]
pub struct ChessGame {
    board: Board,
    king_positions: [Position; 2],
    current_player: Player,
    // rules: Box<dyn ChessRule>
    pub game_id: GameId,
    pub players: [PlayerId; 2],
    player_info: [(); 2],
}

#[derive(Debug)]
enum CheckStatus {
    NotInCheck,
    One(Vec<Position>),
    Multiple,
}

impl ChessGame {
    pub fn new(players: [PlayerId; 2], player_info: [(); 2]) -> Self {
        Self {
            board: Board::default(),
            king_positions: [Position(4, 7), Position(4, 0)],
            current_player: Player::White,
            game_id: rand::random(),
            players,
            player_info,
        }
    }
    pub fn get_moves(&self) -> Vec<Value> {
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
                    if piece.get_player() != self.current_player {
                        moves[y as usize][x as usize] = Some(piece.get_moves(&self.board));
                    }
                }
            }
        }

        let mut is_in_check = CheckStatus::NotInCheck;
        let mut pinned_pieces: HashMap<Position, Vec<Position>> = HashMap::new();
        // all directions that can 'capture' the king
        for direction in CHECKABLE_DIRECTIONS {
            let all_moves = direction.get_all_moves(
                self.king_positions[self.current_player.player_index()],
                self.current_player,
                &self.board,
            );

            'all_moves_loop: for line in all_moves {
                let mut pinned: Option<Position> = None;
                // get moves for each direction
                let mut line_moves = Vec::new();
                for position in line {
                    line_moves.push(position);
                    if let Some(piece) = self.board.get(position) {
                        // if a piece can threaten king (is in his sightline e.g. diagonal)
                        // check which player's it is
                        if piece.get_player() != self.current_player
                            && piece // check if the piece can move in selected direction
                                .get_directions_ids()
                                .contains(&direction.direction_id())
                        {
                            if let Some(pinned_piece) = pinned {
                                pinned_pieces.insert(pinned_piece, line_moves);
                                break;
                            }
                            match is_in_check {
                                CheckStatus::NotInCheck => {
                                    is_in_check = CheckStatus::One(line_moves)
                                }
                                _ => {
                                    is_in_check = CheckStatus::Multiple;
                                    break 'all_moves_loop;
                                }
                            };
                            break;
                        } else if pinned.is_none() {
                            pinned = Some(position);
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        println!("moves: {:?}", moves);
        println!("is in check: {:?}", is_in_check);
        match is_in_check {
            CheckStatus::NotInCheck => {
                for y in 0..8 {
                    for x in 0..8 {
                        match pinned_pieces.get(&Position(x, y)) {
                            Some(legal_moves) => {
                                let mut piece_legal_moves = Vec::new();
                                if let Some(piece_moves) = moves[y as usize][x as usize].take() {
                                    for piece_move in piece_moves {
                                        if legal_moves.contains(&piece_move) {
                                            piece_legal_moves.push(piece_move);
                                        }
                                    }
                                    moves[y as usize][x as usize] = Some(piece_legal_moves);
                                }
                            }
                            None => {
                                // println!("Happens 2");
                            }
                        }
                    }
                }
            }
            CheckStatus::One(legal_moves) => {
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
            }
            CheckStatus::Multiple => {
                moves = [
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                    [None, None, None, None, None, None, None, None],
                ];

                moves[self.king_positions[self.current_player.player_index()].y() as usize]
                    [self.king_positions[self.current_player.player_index()].x() as usize] =
                    Some(vec![Position(0, 0), Position(7, 7)]);
            }
        }
        println!("moves: {:?}", moves);

        let mut final_moves = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.board.get(Position::new(x, y)).as_ref() {
                    final_moves.push(json!({
                        "filename": piece.get_piece_name(),
                        "position": piece.get_position(),
                        "moves": match moves[y as usize][x as usize].take() {
                            Some(legal_moves) => legal_moves,
                            None => Vec::with_capacity(0),
                        },
                    }));
                }
            }
        }
        final_moves
    }

    pub fn move_piece(&mut self, player_id: PlayerId, from: Position, to: Position) -> String {
        self.board.move_piece(from, to);
        self.current_player.change_player();
        if from == self.king_positions[self.current_player.player_index()] {
            self.king_positions[self.current_player.player_index()] = to;
        }
        format!("{} -> {}", from, to)
    }
}
