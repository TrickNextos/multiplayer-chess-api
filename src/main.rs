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
use actors::{game_organizer::GameOrganizer, ws_actions::MessageFromWs};

#[actix::main]
async fn main() -> std::io::Result<()> {
    println!("server started");
    dotenv().ok();

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("No DATABASE_URL found in .env"))
        .await
        .expect("Couldnt make db pool");

    let game_organizer: Recipient<MessageFromWs> = GameOrganizer.start().recipient();

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
            .route("/game/ws", web::get().to(ws::ws))
    })
    .bind(("localhost", 5678))?
    .run()
    .await?;
    Ok(())
}
