use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Unable to connect to database")]
    ConnectionError(DbErr),
    #[error("Environment variables are not set correctly")]
    EnvironmentError,
}

pub async fn establish_connection() -> Result<DatabaseConnection, DbError> {
    let db_url = env::var("DATABASE_URL").or_else(|_| Err(DbError::EnvironmentError))?;

    let res = Database::connect(db_url)
        .await
        .map_err(|e| DbError::ConnectionError(e))?;
    Ok(res)
}
