use serde_json::{json, Value};
use std::collections::HashMap;

use super::{
    board::Board,
    direction::{
        BishopDirection, Direction, KingDirection, KnightDirection, PawnEatingDirection,
        RookDirection,
    },
    Player, Position, PositionWithDirection,
};
use crate::{chess_logic::direction::get_direction_from_id, GameId, PlayerId};

#[derive(Debug)]
pub struct ChessGame {
    pub board: Board,
    king_positions: [Position; 2],
    current_player: Player,
    pub can_enpassant: [Option<Position>; 2],
    // rules: Box<dyn ChessRule>
    pub game_id: GameId,
    pub players: [PlayerId; 2],
    /// either 0 or 1
    pub current_player_id: usize,

    calculated_legal_moves: Option<(bool, [[Option<Vec<PositionWithDirection>>; 8]; 8])>,

    pub current_chat_data: Vec<(PlayerId, String)>,
    pub current_move_data: Vec<String>,

    /// who requested draw (player id)
    pub current_draw_status: Option<usize>,
}

#[derive(Debug)]
pub enum CheckStatus {
    NotInCheck,
    One(Vec<PositionWithDirection>),
    Multiple,
}

const CHECKABLE_DIRECTIONS: [&'static dyn Direction; 5] = [
    &RookDirection(),
    &BishopDirection(),
    &PawnEatingDirection(),
    &KnightDirection(),
    &KingDirection(),
];

impl ChessGame {
    pub fn new(players: [PlayerId; 2]) -> Self {
        Self {
            board: Board::default(),
            king_positions: [Position(4, 7), Position(4, 0)],
            current_player: Player::White,
            game_id: rand::random(),
            can_enpassant: [None; 2],
            players,
            current_player_id: 0,
            calculated_legal_moves: None,
            current_chat_data: Vec::new(),
            current_move_data: Vec::new(),
            current_draw_status: None,
        }
    }
    fn get_moves(&mut self) -> (bool, [[Option<Vec<PositionWithDirection>>; 8]; 8]) {
        if let Some(moves) = self.calculated_legal_moves.take() {
            return moves.clone();
        }

        let mut moves: [[Option<Vec<PositionWithDirection>>; 8]; 8] = [
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None],
        ];

        for y in 0..8 {
            for x in 0..8 {
                if let Some(piece) = self.board.get(Position(x, y)) {
                    if piece.get_player() == self.current_player {
                        moves[y as usize][x as usize] = Some(piece.get_moves(&self.board));
                    }
                }
            }
        }

        // there is an edge case at en passant where the pawn is pinned and can't do en passant as the king will be in check
        enum PinType {
            NotPinned,
            One(Position),
            Two(Position),
        }

        let mut is_in_check = CheckStatus::NotInCheck;
        let mut pinned_pieces: HashMap<Position, Vec<PositionWithDirection>> = HashMap::new();
        let mut cant_enpassant = Vec::new();
        // all directions that can 'capture' the king
        for direction in CHECKABLE_DIRECTIONS {
            let all_moves = direction.get_all_moves(
                self.king_positions[self.current_player.player_index()],
                self.current_player,
                &self.board,
            );
            // println!("NEW DIRECTION {}", direction.direction_id());
            // println!("  dir_moves {all_moves:?}");

            // check kings sightline
            'all_moves_loop: for line in all_moves {
                let mut pinned: PinType = PinType::NotPinned;
                // get moves for each direction
                let mut line_moves = Vec::new();

                // println!("    new line {line:?}");
                for position in line {
                    line_moves.push((position, direction.direction_id()));
                    // println!("  Pos {position}");
                    if let Some(piece) = self.board.get(position) {
                        // println!("Piece on {position}, pinn {pinned:?}");
                        // if a piece can threaten king (is in his sightline e.g. diagonal)
                        // check which player's it is
                        // println!("A piece: {:?}", piece);
                        if piece.get_player() != self.current_player
                            && piece // check if the piece can move in selected direction
                                .get_directions_ids()
                                .contains(&direction.direction_id())
                        {
                            if let PinType::One(pinned_piece) = pinned {
                                pinned_pieces.insert(pinned_piece, line_moves.clone());
                                // println!("    Pinned piece inserted");
                            } else if let PinType::Two(pinned_piece) = pinned {
                                cant_enpassant.push(pinned_piece);
                                cant_enpassant.push(piece.get_position());
                            } else {
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
                            }
                        } else if let PinType::NotPinned = pinned {
                            // println!("    pinned piece added, pos: {position:?}, {position}");
                            pinned = PinType::One(position);
                        } else if let PinType::One(pinned_piece) = pinned {
                            println!("wooooowww");
                            pinned = PinType::Two(pinned_piece);
                        } else {
                            // break if this is the second piece in a straight line
                            break;
                        }
                    }
                }
                // println!("  Exited");
            }
        }

        println!("moves: {:?}", moves);
        println!("is in check: {:?}", is_in_check);
        println!("Pinned_pieces: {:?}", pinned_pieces);
        println!("cant enpassant {:?}", cant_enpassant);

        let current_king_pos: Position = self.king_positions[self.current_player.player_index()];
        match is_in_check {
            CheckStatus::NotInCheck => {
                for y in 0..8 {
                    for x in 0..8 {
                        match pinned_pieces.get(&Position(x, y)) {
                            Some(legal_moves) => {
                                if let Some(piece_moves) = moves[y as usize][x as usize].take() {
                                    moves[y as usize][x as usize] = Some(
                                        piece_moves
                                            .into_iter()
                                            .filter(|piece_move| {
                                                legal_moves.iter().any(|mv| mv.0 == piece_move.0)
                                            })
                                            .collect(),
                                    );
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
            CheckStatus::One(legal_moves) => {
                println!("I AM IN CHEEEECK");
                for y in 0..8 {
                    for x in 0..8 {
                        if current_king_pos == Position::new(x, y) {
                            continue;
                        }
                        if let Some(piece_moves) = moves[y as usize][x as usize].take() {
                            moves[y as usize][x as usize] = Some(
                                piece_moves
                                    .into_iter()
                                    .filter(|piece_move| {
                                        legal_moves.iter().any(|mv| mv.0 == piece_move.0)
                                    })
                                    .collect(),
                            );
                        }
                    }
                }
            }
            CheckStatus::Multiple => {
                println!("mult check");
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

                let king_pos = self.king_positions[self.current_player.player_index()];
                if let Some(p) = self.board.get(king_pos).take() {
                    moves[king_pos.y() as usize][king_pos.x() as usize] =
                        Some(p.get_moves(&self.board));
                }
            }
        }
        // println!("moves: {:?}", moves);

        //moves for king are a bit special
        let king_pos = self.king_positions[self.current_player.player_index()];
        let mut legal_king_moves = Vec::new();
        if let Some(king_moves) = moves[king_pos.y() as usize][king_pos.x() as usize].take() {
            for king_move in king_moves {
                // check if any enemies can 'see' this square
                if check_if_valid_king_pos(&self.board, self.current_player, king_move.0, king_pos)
                {
                    legal_king_moves.push(king_move);
                    // println!("HAPPEAUSYDGAIWUGYD");
                }
            }
        }

        moves[king_pos.y() as usize][king_pos.x() as usize] = Some(legal_king_moves);
        // println!("moves 2: {:?}", moves);

        let mut has_moves = false;
        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = &self.board.get(Position::new(x, y)) {
                    moves[y as usize][x as usize] =
                        moves[y as usize][x as usize].take().and_then(|mv| {
                            Some({
                                let mvs: Vec<_> = mv
                                    .into_iter()
                                    .filter(|(to, direction_id)| {
                                        get_direction_from_id(*direction_id).extra_req()(
                                            self,
                                            (piece.get_position(), *direction_id),
                                            (*to, *direction_id),
                                            piece.get_player(),
                                            &self.board,
                                            &moves,
                                            &pinned_pieces,
                                            &cant_enpassant,
                                        )
                                        .map_or(false, |t| t)
                                    })
                                    .collect();
                                if mvs.len() > 0 {
                                    has_moves = true;
                                }
                                mvs
                            })
                        });
                }
            }
        }

        self.calculated_legal_moves = Some((has_moves, moves.clone()));
        println!("has moves: {}", has_moves);
        (has_moves, moves)
    }

    pub fn get_moves_as_json(&mut self) -> Vec<Value> {
        let mut moves = self.get_moves();
        let mut final_moves = Vec::new();

        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.board.get(Position::new(x, y)).as_ref() {
                    final_moves.push(json!({
                        "filename": piece.get_piece_name(),
                        "position": piece.get_position(),
                        "moves": match moves.1[y as usize][x as usize].take() {
                            Some(legal_moves) => legal_moves.into_iter().map(|(to, _dir_id)| to).collect::<Vec<Position>>(),
                            None => Vec::with_capacity(0),
                        },
                    }));
                }
            }
        }

        final_moves
    }

    pub fn get_position_as_json(&mut self) -> Vec<Value> {
        let mut final_moves = Vec::new();

        for x in 0..8 {
            for y in 0..8 {
                if let Some(piece) = self.board.get(Position::new(x, y)).as_ref() {
                    final_moves.push(json!({
                        "filename": piece.get_piece_name(),
                        "position": piece.get_position(),
                        "moves": []
                    }));
                }
            }
        }

        final_moves
    }

    pub fn move_piece(&mut self, from: Position, to: Position) -> Result<(bool, String), ()> {
        let piece_filename = self.board.get(from).ok_or(())?.get_filename();
        let piece_player = self.board.get(from).ok_or(())?.get_player();
        let direction_id = self.get_moves().1[from.y() as usize][from.x() as usize]
            .clone()
            .ok_or(())?
            .into_iter()
            .filter(|legal_move| legal_move.0 == to)
            .collect::<Vec<(Position, i32)>>()
            .get(0)
            .ok_or(())?
            .1;

        // used for notation, check if 2 pieces of same type can move to same square
        let mut extra_info = {
            let can_move_to_square: Vec<Position> = self
                .board
                .get(from)
                .ok_or(())?
                .get_directions_ids()
                .iter()
                .map(|id| {
                    get_direction_from_id(*id)
                        .get_all_moves(to, piece_player.opponent(), &self.board)
                        .into_iter()
                        .flatten()
                        .filter_map(|mv| self.board.get(mv))
                        .filter(|p| {
                            p.get_filename() == piece_filename && p.get_player() == piece_player
                        })
                        .map(|p| p.get_position())
                })
                .flatten()
                .collect();
            if can_move_to_square.len() > 1 {
                from.to_string()
                    .chars()
                    .next()
                    .expect("There are 2 chars here")
            } else {
                '\0'
            }
        };

        // move the actual piece and make side effects (e.g. castle, en passant)
        let was_capture = self.board.move_piece(from, to);
        get_direction_from_id(direction_id).side_effect(from, to, piece_player, self);

        // notation
        if was_capture && piece_filename == "p" {
            extra_info = from
                .to_string()
                .chars()
                .next()
                .expect("There are 2 chars here");
        }

        // reset some stuff
        self.can_enpassant[self.current_player.player_index()] = None;
        self.calculated_legal_moves = None;

        // track kings pos
        if from == self.king_positions[self.current_player.player_index()] {
            self.king_positions[self.current_player.player_index()] = to;
        }

        // it's other players turn
        self.current_player.change_player();
        self.current_player_id = (self.current_player_id + 1) % 2;

        // make notation
        // TODO: add check and checkmate notation (+ and #)
        Ok((
            self.get_moves().0,
            format!(
                "{}{extra_info}{}{to}",
                match piece_filename {
                    "p" => String::new(),
                    other => other.to_uppercase(),
                },
                {
                    if was_capture {
                        "x"
                    } else {
                        ""
                    }
                }
            ),
        ))
    }
}

pub fn check_if_valid_king_pos(
    board: &Board,
    current_player: Player,
    king_move: Position,
    king_pos: Position,
) -> bool {
    for direction in CHECKABLE_DIRECTIONS {
        'current_dirrection: for line in direction.get_all_moves(king_move, current_player, board) {
            for pos in line {
                if let Some(p) = board.get(pos) {
                    if pos == king_pos {
                        continue;
                    }
                    if p.get_directions_ids().contains(&direction.direction_id())
                        && p.get_player() != current_player
                    {
                        return false;
                    }
                    continue 'current_dirrection;
                }
            }
        }
    }
    true
}
