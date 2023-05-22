use rocket::{http::Status, response::Responder, Response};
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Unable to connect to database")]
    ConnectionError(String, DbErr),
    #[error("Environment variables are not set correctly")]
    EnvironmentError,
}

impl<'r> Responder<'r, 'static> for DbError {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build().status(Status::InternalServerError).ok()
    }
}

pub async fn establish_connection() -> Result<DatabaseConnection, DbError> {
    let db_url = env::var("DATABASE_URL").map_err(|_| DbError::EnvironmentError)?;

    let res = Database::connect(db_url.to_owned())
        .await
        .map_err(|e| DbError::ConnectionError(db_url, e))?;
    Ok(res)
}

pub async fn redis_connection() -> anyhow::Result<redis::Client> {
    let redis_url = env::var("REDIS_URL").map_err(|e| anyhow::anyhow!(e))?;
    let client = redis::Client::open(redis_url).map_err(|e| anyhow::anyhow!(e))?;

    Ok(client)
}
