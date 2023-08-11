use redis::{aio::Connection as RedisConnection, AsyncCommands};
use rocket::{http::Status, response::Responder, Response};
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::env;
use thiserror::Error;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RedisRefresh: Send {
    async fn get_item(
        &mut self,
        key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;

    async fn set_item(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn delete_item(
        &mut self,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl RedisRefresh for RedisConnection {
    async fn get_item(
        &mut self,
        key: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.get::<&str, Option<String>>(key).await?)
    }

    async fn set_item(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.set::<&str, &str, ()>(key, value).await?;
        Ok(())
    }

    async fn delete_item(
        &mut self,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.del::<&str, ()>(key).await?;
        Ok(())
    }
}

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

#[cfg(test)]
pub mod test {
    use migration::MigratorTrait;
    use sea_orm::{Database, DatabaseConnection, DbErr};

    pub async fn establish_connection() -> Result<DatabaseConnection, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        migration::Migrator::up(&db, None).await?;

        Ok(db)
    }
}
