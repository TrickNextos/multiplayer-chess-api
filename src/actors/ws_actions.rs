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
    Chat(String),
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub struct MessageFromWs {
    pub id: WsPlayer,
    pub data: MessageFromWsType,
}

#[derive(Deserialize, Debug)]
pub struct OuterMessageFromWs {
    pub game: Option<u32>,
    pub data: String, // later to be deserialized into MessageFromWs
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
    game_id: String,
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
            "chat" => MessageFromWsType::Chat(input.data.to_owned()),
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
    Chat(String),
    MoveInfo(String),
}

#[derive(Debug, Deserialize)]
pub struct PlayerData {
    pub username: String,
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub enum DataToWs {
    Message(u32, MessageToWs),
    Init(u32, PlayerData, Recipient<MessageFromWs>),
    End(GameEnded),
}

impl MessageToWs {
    pub fn serialize(&self, game_id: u32) -> String {
        match self {
            Self::Moves(vec) => serde_json::to_string(&OutputData {
                action: "move".to_owned(),
                data: serde_json::to_string(vec).expect("Moves vec to string shuold never fail"),
                game_id: game_id.to_string(),
            })
            .unwrap(),
            Self::Chat(text) => serde_json::to_string(&OutputData {
                action: "chat".to_owned(),
                data: format!("<b>Opponent: </b>{}", text.to_owned()),
                game_id: game_id.to_string(),
            })
            .unwrap(),
            Self::MoveInfo(text) => serde_json::to_string(&OutputData {
                action: "move info".to_owned(),
                data: text.to_owned(),
                game_id: game_id.to_string(),
            })
            .unwrap(),
        }
    }
}
