use actix::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FromToPosition {
    from: [usize; 2],
    to: [usize; 2],
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub enum MessageFromWs {
    Move(FromToPosition),
    Premove(FromToPosition),
}

#[derive(Debug, Deserialize)]
struct InputData {
    data: String,
    action: String,
}

#[derive(Debug, Serialize)]
struct ErrorMessage {
    error: &'static str,
}

impl MessageFromWs {
    pub fn deserialize(string: &str) -> Result<Self, String> {
        let input = match serde_json::from_str::<InputData>(string) {
            Ok(input_data) => input_data,
            Err(_e) => return Err(json!({"error": "badData"}).to_string()),
        };

        Ok(match input.action.as_str() {
            "move" => MessageFromWs::Move(match serde_json::from_str(input.data.as_str()) {
                Ok(n) => n,
                Err(_e) => return Err(json!({"error": "badData"}).to_string()),
            }),
            "premove" => MessageFromWs::Premove(match serde_json::from_str(input.data.as_str()) {
                Ok(n) => n,
                Err(_e) => return Err(json!({"error": "badData"}).to_string()),
            }),
            _ => unreachable!("Update API or UI (wrong action code sent throught ws)"),
        })
    }
}
