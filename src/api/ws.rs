use actix::Recipient;
use actix_web::{web, Error as ActixWebError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::actors::{game::ChessGameWs, ws_actions::MessageFromWs};

// TODO: add heartbeat functionality
pub async fn ws(
    game_organizer: web::Data<Recipient<MessageFromWs>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, ActixWebError> {
    ws::start(
        ChessGameWs::new(vec![1usize, 3usize], game_organizer.into_inner()),
        &req,
        stream,
    )
}
