use actix::{Actor, Recipient};
use actix_cors::Cors;
use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;

mod api;
use api::{auth, healthcheck, ws};

mod extractors;

mod actors;
use actors::game_organizer::{AddNewPlayer, GameOrganizer};

mod chess_logic;
use chess_logic::{Board, Position};

#[actix::main]
async fn main() -> std::io::Result<()> {
    println!("server started");
    dotenv().ok();

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("No DATABASE_URL found in .env"))
        .await
        .expect("Couldnt make db pool");

    let game_organizer: Recipient<AddNewPlayer> = GameOrganizer::default().start().recipient();

    let board: Board = Board::from_fen("8/8/8/4R3/8/8/8/8 w QKqk - 0 0").unwrap();
    let piece = board.get(Position::new(4, 3));
    println!("{:?}", piece);
    if let Some(piece) = piece {
        let moves = piece.get_moves(&board);
        println!("moves: {:?}", moves);
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(Data::new(
                std::env::var("JWT_TOKEN_SECRET").expect("No JWT_TOKEN_SECRET found in .env"),
            ))
            .app_data(Data::new(db_pool.clone()))
            .app_data(Data::new(game_organizer.clone()))
            .service(auth::login_scope())
            .route("/healthcheck", web::get().to(healthcheck))
            .route("/game/ws/{id}", web::get().to(ws::ws))
    })
    .bind(("localhost", 5678))?
    .run()
    .await?;
    Ok(())
}
