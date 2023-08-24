use actix_cors::Cors;
use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use dotenv::dotenv;
use sqlx::mysql::MySqlPoolOptions;

mod api;
use api::{auth, healthcheck};

mod extractors;

#[actix::main]
async fn main() -> std::io::Result<()> {
    println!("server started");
    dotenv().ok();

    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("No DATABASE_URL found in .env"))
        .await
        .expect("Couldnt make db pool");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(Data::new(
                std::env::var("JWT_TOKEN_SECRET").expect("No JWT_TOKEN_SECRET found in .env"),
            ))
            .app_data(Data::new(db_pool.clone()))
            .service(auth::login_scope())
            .route("/healthcheck", web::get().to(healthcheck))
    })
    .bind(("localhost", 5678))?
    .run()
    .await?;
    Ok(())
}
