use crate::{
    chess_logic::{Player, Position},
    PlayerId,
};
use actix_web::{
    web::{self, Path},
    HttpRequest, HttpResponse,
};
use actix_ws::Message;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc::{self, Sender};

use crate::game_organizer::GameOrganizerRequest;

#[derive(Debug, Deserialize)]
pub struct WsMessageIncoming {
    action: String,
    game_id: Option<u32>,
    data: Value,
}

#[derive(Debug, Clone)]
enum WsAction {
    Move {
        from: Position,
        to: Position,
    },
    Chat(String),
    #[allow(dead_code)]
    End(ChessEnd),
    NewGame(NewGameOptions),
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NewGameOptions {
    pub prefered_color: Option<Player>,
    pub opponent: Option<PlayerId>,
    pub game_type: SingleplayerMultiplayer,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SingleplayerMultiplayer {
    Singleplayer,
    Multiplayer,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub enum ChessEnd {
    DrawAsk,
    DrawConfirm,
    DrawCancel,
    Resign,
}

pub async fn game_ws(
    req: HttpRequest,
    body: web::Payload,
    path: Path<usize>,
    game_organizer: web::Data<Sender<GameOrganizerRequest>>,
) -> HttpResponse {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, body).expect("neki");
    let id = path.into_inner();
    actix_rt::spawn(async move {
        use GameOrganizerRequest::*;
        let (tx, mut rx) = mpsc::channel(32);
        println!("Connection started: {id}");

        let _ = game_organizer.send(Connect(id, tx)).await;
        loop {
            tokio::select! {
                Some(Ok(msg)) = msg_stream.recv() => {
                    match msg {
                        Message::Text(msg) => {

                            if let Ok(Some((game_id, msg))) = deserialize_ws_msg(msg.to_string().as_str()) {
                                match msg {
                                    WsAction::Move { from, to } => {
                                        let _ = game_organizer.send(Move(id, game_id, from, to)).await;
                                    }
                                    WsAction::Chat(text) => {
                                        let _ = game_organizer.send(Chat(id, game_id, text)).await;
                                    }
                                    WsAction::End(reason) => {
                                        let _ = game_organizer.send(End(id, game_id, reason)).await;
                                    }
                                    WsAction::NewGame(options) => {
                                        let _ = game_organizer.send(NewGame(id, options)).await;
                                    }
                                }
                            }
                        else {
                            println!("{:?}", deserialize_ws_msg(msg.to_string().as_str()));
                        }
                        }
                        Message::Pong(_) => {
                            let _ = session.ping(b"").await;
                        }
                        _ => break,
                    };
                }
                Some(msg) = rx.recv() => {
                    let _ = session.text(msg).await;
                }
            }
        }
        let _ = game_organizer.send(GameOrganizerRequest::Close(id)).await;
        println!("Client connection closed {id}");
    });

    response
}

fn deserialize_ws_msg(msg: &str) -> Result<Option<(u32, WsAction)>, serde_json::Error> {
    let message: WsMessageIncoming = serde_json::from_str(msg)?;

    Ok(Some(match message.action.as_str() {
        "move" => (
            message.game_id.unwrap(),
            WsAction::Move {
                from: serde_json::from_value(message.data["from"].clone())?,
                to: serde_json::from_value(message.data["to"].clone())?,
            },
        ),
        "chat" => (
            message.game_id.unwrap(),
            WsAction::Chat(serde_json::from_value(message.data)?),
        ),
        "new_game" => (0, WsAction::NewGame(serde_json::from_value(message.data)?)),
        "end" => (
            message.game_id.unwrap(),
            WsAction::End(match serde_json::from_value::<ChessEnd>(message.data) {
                Ok(n) => n,
                Err(_) => {
                    println!("wrong chess-end type");
                    return Ok(None);
                }
            }),
        ),
        _ => {
            println!("wrong error code");
            return Ok(None);
        }
    }))
}
