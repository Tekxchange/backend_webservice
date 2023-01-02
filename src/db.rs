use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Unable to connect to database")]
    ConnectionError,
}

pub fn establish_connection() -> Result<PgConnection, DbError> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").or_else(|_| Err(DbError::ConnectionError))?;
    Ok(PgConnection::establish(&db_url).or_else(|_| Err(DbError::ConnectionError))?)
}
