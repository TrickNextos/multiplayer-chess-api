use actix::Recipient;
use actix_web::{
    web::{self, Path},
    Error as ActixWebError, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

use crate::actors::{game_organizer::AddNewPlayer, game_ws::ChessGameWs};

// TODO: add heartbeat functionality
pub async fn ws(
    game_organizer: web::Data<Recipient<AddNewPlayer>>,
    req: HttpRequest,
    stream: web::Payload,
    id: Path<usize>,
    // token: AuthenticationToken,
) -> Result<HttpResponse, ActixWebError> {
    println!("connection made with {}", id);
    ws::start(
        ChessGameWs::new(*id, game_organizer.into_inner()),
        &req,
        stream,
    )
}
