use crate::chess_logic::Position;
use actix_web::{
    web::{self, Path},
    HttpRequest, HttpResponse,
};
use actix_ws::Message;
use serde::Deserialize;
use tokio::sync::mpsc::{self, Sender};

use crate::game_organizer::GameOrganizerRequest;

#[derive(Debug, Deserialize)]
pub struct WsMessageIncoming {
    action: String,
    game_id: Option<u32>,
    data: String,
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
    NewGame,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
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
        let _ = game_organizer.send(Connect(id, tx)).await;

        loop {
            tokio::select! {
                Some(Ok(msg)) = msg_stream.recv() => {
                // println!("Happens {:?}", msg);
                    match msg {
                        Message::Text(msg) => {

                            if let Ok(Some((game_id, msg))) = deserialize_ws_msg(msg.to_string().as_str()) {
                                // println!("Happens1 {:?}", msg);
                                // dbg!(game_id);
                                match msg {
                                    WsAction::Move { from, to } => {
                                        let _ = game_organizer.send(Move(id, game_id, from, to)).await;
                                    }
                                    WsAction::Chat(text) => {
                                        let _ = game_organizer.send(Chat(id, game_id, text)).await;
                                    }
                                    WsAction::End(thing) => {
                                        let _ = game_organizer.send(End(id, game_id, thing)).await;
                                    }
                                    WsAction::NewGame => {
                                        let _ = game_organizer.send(NewGame(id)).await;
                                    }
                                }
                            }
                        }
                        Message::Pong(_) => {
                            let _ = session.ping(b"").await;
                        }
                        _ => break,
                    };
                }
                Some(msg) = rx.recv() => {
                    // dbg!(msg.clone());
                    let _ = session.text(msg).await;
                }
            }
        }
        let _ = game_organizer.send(GameOrganizerRequest::Close(id)).await;
        println!("Client connection closed");
    });

    response
}

fn deserialize_ws_msg(msg: &str) -> Result<Option<(u32, WsAction)>, serde_json::Error> {
    let message: WsMessageIncoming = serde_json::from_str(msg)?;

    Ok(Some(match message.action.as_str() {
        "move" => {
            #[derive(Debug, Deserialize)]
            struct FromTo {
                from: Position,
                to: Position,
            }
            let inner: FromTo = serde_json::from_str(message.data.as_str())?;
            (
                message.game_id.unwrap(),
                WsAction::Move {
                    from: inner.from,
                    to: inner.to,
                },
            )
        }
        "chat" => (message.game_id.unwrap(), WsAction::Chat(message.data)),
        "new_game" => (0, WsAction::NewGame),
        _ => {
            println!("wrong error code");
            return Ok(None);
        }
    }))
}
