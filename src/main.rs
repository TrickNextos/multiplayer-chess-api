use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;

mod api;
use api::{auth, game_ws, healthcheck, social};

mod chess_logic;
mod extractors;
mod game_organizer;
mod sql;

pub type PlayerId = usize;
pub type GameId = u32;
pub type WsMessageOutgoing = String;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("server starting");
    dotenv().ok();

    std::fs::create_dir_all("../games/")?;

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("No DATABASE_URL found in .env"))
        .await
        .expect("Couldnt make db pool");

    // let board: Board = Board::from_fen("8/8/8/4R3/8/8/8/8 w QKqk - 0 0").unwrap();
    // let piece = board.get(Position::new(4, 3));
    // println!("{:?}", piece);
    // if let Some(piece) = piece {
    //     let moves = piece.get_moves();
    //     println!("moves: {:?}", moves);
    // }

    let game_organizer = Data::new(game_organizer::GameOrganizer::new(db_pool.clone()));

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(Logger::default())
            .app_data(Data::new(
                std::env::var("JWT_TOKEN_SECRET").expect("No JWT_TOKEN_SECRET found in .env"),
            ))
            .app_data(Data::new(db_pool.clone()))
            .app_data(game_organizer.clone())
            .service(auth::login_scope())
            .service(social::social_scope())
            .route("/healthcheck", web::get().to(healthcheck))
            .route("/game/ws/{id}", web::get().to(game_ws::game_ws))
    })
    .bind(("0.0.0.0", 5678))?
    .run()
    .await?;
    Ok(())
}
