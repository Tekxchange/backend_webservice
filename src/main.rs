#[macro_use]
extern crate rocket;
mod catchers;
mod controllers;
mod cors;
mod db;
mod dtos;
mod guards;
mod logger;
mod models;
mod services;
mod statsd;
use cors::{Cors, Options};
use logger::setup_loki;
use migration::{Migrator, MigratorTrait};
use rocket::{response::Responder, Response};
use serde_json::json;
use services::{AuthService, UserService};
use statsd::Statsd;
use std::env;

use crate::models::user::UserRegister;

#[derive(Debug)]
pub struct AnyhowResponder(anyhow::Error);

impl<'r> Responder<'r, 'static> for AnyhowResponder {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let inner_error = self.0.to_string();
        Response::build_from(json!({ "error": inner_error }).respond_to(request)?).ok()
    }
}

impl std::fmt::Display for AnyhowResponder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let self_str = self.0.to_string();
        write!(f, "{self_str}")
    }
}

#[launch]
pub async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    setup_loki();
    let conn = db::establish_connection().await.unwrap();
    let redis = db::redis_connection().await.unwrap();
    let key = AuthService::get_key_pair().unwrap();

    Migrator::up(&conn, None).await.unwrap();

    let user_service = UserService::new(conn.clone());
    let found_admin = user_service
        .username_exists(crate::models::user::ADMIN_USERNAME)
        .await
        .unwrap();

    if !found_admin {
        println!("No admin found -- Seeding new admin");
        let admin_email = env::var("ADMIN_EMAIL").unwrap();
        let admin_password = env::var("ADMIN_PASSWORD").unwrap();
        let user_register = UserRegister {
            email: admin_email,
            password: admin_password,
            username: crate::models::user::ADMIN_USERNAME.to_owned(),
        };
        let user_id = user_service.create_user(user_register, true).await.unwrap();
        user_service
            .update_role_for_user(user_id, models::role::Role::Admin)
            .await
            .unwrap();
    }

    controllers::mount_routes(rocket::build())
        .manage(conn)
        .manage(redis)
        .manage(key)
        .attach(Statsd::default())
        .attach(Cors)
        .attach(Options)
        .register(
            "/",
            catchers![
                catchers::not_found,
                catchers::unauthorized,
                catchers::internal_error,
                catchers::unprocessable
            ],
        )
}

#[cfg(test)]
use sea_orm::DatabaseConnection;
#[cfg(test)]
pub async fn create_rocket_instance(
    memory_conn: Option<DatabaseConnection>,
) -> anyhow::Result<rocket::Rocket<rocket::Build>> {
    use db::test::establish_connection;
    let key = AuthService::get_key_pair()?;

    let memory_conn = memory_conn.unwrap_or(establish_connection().await?);
    Ok(controllers::mount_routes(rocket::build())
        .manage(memory_conn)
        .manage(key)
        .attach(Cors)
        .attach(Options)
        .register(
            "/",
            catchers![
                catchers::not_found,
                catchers::unauthorized,
                catchers::internal_error,
                catchers::unprocessable
            ],
        ))
}
