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
use rocket::{response::Responder, Config, Response};
use serde_json::json;
use services::{AuthService, UserService};
use statsd::Statsd;
use std::env;

use crate::models::user::UserRegister;

#[derive(Debug)]
pub struct AnyhowResponder(anyhow::Error);

impl<'r> Responder<'r, 'static> for AnyhowResponder {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let (method, uri) = request
            .route()
            .map(|r| (r.method.as_str(), r.uri.as_str()))
            .unzip();

        let trace = self.0.backtrace();
        let source = self.0.root_cause().to_string();

        tracing::error!(
            error = self.0.to_string(),
            backtrace = trace.to_string(),
            source,
            method,
            uri
        );
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
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "none");
    env::set_var("rust_tekxchange_backend", "debug");
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
        tracing::info!("No admin found -- Seeding new admin");
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

    let mut config = Config::default();
    config.cli_colors = false;
    config.log_level = rocket::log::LogLevel::Off;

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
