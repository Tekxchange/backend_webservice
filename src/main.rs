#[macro_use]
extern crate rocket;
mod controllers;
mod db;
mod models;
mod services;
use migration::{Migrator, MigratorTrait};

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let conn = db::establish_connection().await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    controllers::mount_routes(rocket::build())
}
