use actix::{Message, Recipient};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::chess_logic::Position;

use super::{game::GameEnded, WsPlayer};

#[derive(Debug, Deserialize)]
pub struct FromToPosition {
    pub from: Position,
    pub to: Position,
}

#[derive(Debug)]
pub enum MessageFromWsType {
    Move(FromToPosition),
    Premove(FromToPosition),
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub struct MessageFromWs {
    pub id: WsPlayer,
    pub data: MessageFromWsType,
}

impl MessageFromWs {
    pub fn new_message(id: WsPlayer, msg: &str) -> Result<Self, String> {
        Ok(Self {
            id,
            data: MessageFromWsType::deserialize(msg)?,
        })
    }
}

#[derive(Debug, Deserialize)]
struct InputData {
    data: String,
    action: String,
}

// TODO: maybe make this take MessageToWs instead of String
#[derive(Debug, Serialize)]
struct OutputData {
    action: String,
    data: String,
}

impl MessageFromWsType {
    pub fn deserialize(string: &str) -> Result<Self, String> {
        let input = match serde_json::from_str::<InputData>(string) {
            Ok(input_data) => input_data,
            Err(_e) => return Err(json!({"error": "badData"}).to_string()),
        };

        Ok(match input.action.as_str() {
            "move" => MessageFromWsType::Move(match serde_json::from_str(input.data.as_str()) {
                Ok(n) => n,
                Err(_e) => return Err(json!({"error": "badData"}).to_string()),
            }),
            "premove" => {
                MessageFromWsType::Premove(match serde_json::from_str(input.data.as_str()) {
                    Ok(n) => n,
                    Err(_e) => return Err(json!({"error": "badData"}).to_string()),
                })
            }
            _ => unreachable!("Update API or UI (wrong action code sent throught ws)"),
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PieceWithMoves {
    filename: String,
    position: Position,
    moves: Vec<Position>,
}

impl PieceWithMoves {
    pub fn new(filename: String, position: Position, moves: Vec<Position>) -> Self {
        Self {
            filename,
            position,
            moves,
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub enum MessageToWs {
    Moves(Vec<PieceWithMoves>),
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub enum DataToWs {
    Message(MessageToWs),
    Init(Recipient<MessageFromWs>),
    End(GameEnded),
}

impl MessageToWs {
    pub fn serialize(&self) -> String {
        match self {
            Self::Moves(vec) => serde_json::to_string(&OutputData {
                action: "move".to_owned(),
                data: serde_json::to_string(vec).expect("Moves vec to string shuold never fail"),
            })
            .unwrap(),
        }
    }
}
