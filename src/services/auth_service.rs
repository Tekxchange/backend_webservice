use crate::{
    dtos::auth::LoginReturn,
    models::{role::Role, user::UserJwtDto},
    AnyhowResponder,
};
use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use entity::{
    self,
    refresh_token::{self, ActiveModel as RefreshActiveModel, Entity as RefreshEntity},
    user::Model as UserModel,
};
use jwt_simple::prelude::*;
use redis::AsyncCommands;
use redis::{aio::Connection as RedisConnection, Client as RedisClient};
use rocket::{
    outcome::{try_outcome, IntoOutcome},
    request::FromRequest,
    Request, State,
};
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::*, ActiveValue};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    time::Duration,
};
use thiserror::Error;

#[cfg(test)]
mod test;

const KEY_LOCATION: &str = "./auth.key";

#[derive(Error, Debug, Responder)]
pub enum AuthServiceError {
    #[error("An unknown error has occurred")]
    #[response(status = 500)]
    InternalError(AnyhowResponder),
    #[error("Unable to log in.")]
    #[response(status = 401)]
    LoginError(AnyhowResponder),
    #[error("Invalid token")]
    #[response(status = 401)]
    InvalidJWT(AnyhowResponder),
    #[error("Missing refresh token")]
    #[response(status = 401)]
    MissingRefreshToken(AnyhowResponder),
}

pub struct AuthService {
    db: DatabaseConnection,
    redis: Box<dyn RedisRefresh>,
    signing_key: Ed25519KeyPair,
}

#[async_trait]
impl<'r> FromRequest<'r> for AuthService {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let db: &State<DatabaseConnection> =
            try_outcome!(request.guard::<&State<DatabaseConnection>>().await);

        let redis: &State<RedisClient> = try_outcome!(request.guard::<&State<RedisClient>>().await);
        let key: &State<Ed25519KeyPair> =
            try_outcome!(request.guard::<&State<Ed25519KeyPair>>().await);

        redis
            .inner()
            .get_async_connection()
            .await
            .ok()
            .map(|r| Self::new(db.inner().clone(), Box::new(r), key.inner().clone()))
            .or_forward(())
    }
}

impl AuthService {
    pub fn new(db: DatabaseConnection, redis: Box<dyn RedisRefresh>, key: Ed25519KeyPair) -> Self {
        Self {
            db,
            redis,
            signing_key: key,
        }
    }

    pub fn get_key_pair() -> Result<Ed25519KeyPair, AuthServiceError> {
        let save_key = |key: &Ed25519KeyPair| -> Result<(), AuthServiceError> {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(KEY_LOCATION)
                .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

            file.write(&key.to_bytes())
                .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

            Ok(())
        };
        let key = (|| -> Result<Ed25519KeyPair, AuthServiceError> {
            match OpenOptions::new().read(true).open(KEY_LOCATION) {
                Ok(mut file) => {
                    let file_size = file
                        .metadata()
                        .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?
                        .len() as usize;

                    let mut file_buffer = vec![0u8; file_size];
                    file.read(&mut file_buffer).map_err(|e| {
                        AuthServiceError::InternalError(AnyhowResponder(anyhow!(e)))
                    })?;

                    if let Ok(key) = Ed25519KeyPair::from_bytes(&file_buffer) {
                        Ok(key)
                    } else {
                        let key = Ed25519KeyPair::generate();
                        save_key(&key)?;
                        Ok(key)
                    }
                }
                Err(_) => {
                    let key = Ed25519KeyPair::generate();
                    save_key(&key)?;
                    Ok(key)
                }
            }
        })()?;

        Ok(key)
    }

    pub fn hash_password(password: &str) -> Result<String, AuthServiceError> {
        let salt = SaltString::generate(OsRng);
        let argon = Argon2::default();

        let hashed = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(hashed.to_string())
    }

    pub fn verify_password(
        encoded_password: &str,
        raw_password: &str,
    ) -> Result<bool, AuthServiceError> {
        let argon = Argon2::default();
        let hash = PasswordHash::new(encoded_password)
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        match argon.verify_password(raw_password.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub async fn generate_refresh_token(
        &mut self,
        user: &UserModel,
    ) -> Result<String, AuthServiceError> {
        let mut res = self.redis.get_item(&user.id.to_string()).await.unwrap();

        if res.is_none() {
            res = RefreshEntity::find()
                .filter(refresh_token::Column::UserId.eq(user.id))
                .one(&self.db)
                .await
                .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?
                .map(|r| r.token);
            if let Some(ref token) = res {
                self.redis
                    .set_item(&user.id.to_string(), token)
                    .await
                    .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;
            }
        }

        if let Some(token) = res {
            return Ok(token);
        }

        let token = uuid::Uuid::new_v4().to_string();

        RefreshActiveModel {
            token: ActiveValue::Set(token.clone()),
            user_id: ActiveValue::Set(user.id),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        self.redis
            .set_item(&user.id.to_string(), &token)
            .await
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(token)
    }

    pub async fn validate_refresh_token(
        &mut self,
        user_id: i64,
    ) -> Result<Option<String>, AuthServiceError> {
        let mut found = self
            .redis
            .get_item(&user_id.to_string())
            .await
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        if found.is_none() {
            found = RefreshEntity::find()
                .filter(refresh_token::Column::UserId.eq(user_id))
                .one(&self.db)
                .await
                .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?
                .map(|r_token| r_token.token);

            if let Some(ref token) = found {
                self.redis
                    .set_item(&user_id.to_string(), token)
                    .await
                    .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;
            }
        }

        Ok(found)
    }

    pub async fn generate_jwt(
        &mut self,
        user: &UserJwtDto,
        refresh: &str,
        validity: Option<std::time::Duration>,
    ) -> Result<String, AuthServiceError> {
        let claims = Claims::with_custom_claims(
            user.clone(),
            validity
                .unwrap_or_else(|| Duration::from_secs(60 * 60 * 24 * 7))
                .into(),
        );

        let stored_refresh = self.validate_refresh_token(user.id).await?;
        if let None = stored_refresh {
            return Err(AuthServiceError::MissingRefreshToken(AnyhowResponder(
                anyhow!("Missing refresh token"),
            )));
        }
        let stored_refresh = stored_refresh.unwrap();
        if stored_refresh != refresh {
            return Err(AuthServiceError::MissingRefreshToken(AnyhowResponder(
                anyhow!("Mismatched refresh token"),
            )));
        }

        let token = self
            .signing_key
            .sign(claims)
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(token)
    }

    pub fn validate_jwt(
        &self,
        jwt: String,
        tolerance: Option<Duration>,
    ) -> Result<UserJwtDto, AuthServiceError> {
        let claims = self
            .signing_key
            .public_key()
            .verify_token::<UserJwtDto>(
                &jwt,
                Some(VerificationOptions {
                    time_tolerance: Some(tolerance.unwrap_or(Duration::new(0, 0)).into()),
                    ..Default::default()
                }),
            )
            .map_err(|e| AuthServiceError::InvalidJWT(AnyhowResponder(anyhow!(e))))?;

        Ok(claims.custom)
    }

    pub async fn revoke_refresh_token(&mut self, user: &UserModel) -> Result<(), AuthServiceError> {
        let user_id_str = user.id.to_string();

        self.redis
            .delete_item(&user_id_str)
            .await
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        RefreshEntity::delete_many()
            .filter(refresh_token::Column::UserId.eq(user.id))
            .exec(&self.db)
            .await
            .map_err(|e| AuthServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(())
    }

    pub async fn login(
        &mut self,
        potential_password: String,
        user: &UserModel,
    ) -> Result<LoginReturn, AuthServiceError> {
        if !Self::verify_password(&user.password, &potential_password)? {
            return Err(AuthServiceError::LoginError(AnyhowResponder(anyhow!(
                "Unable to validate password"
            ))));
        }

        let user_jwt = UserJwtDto {
            id: user.id,
            role: Role::try_from(user.role).map_err(|_| {
                AuthServiceError::InternalError(AnyhowResponder(anyhow!(
                    "Unable to convert `i64` to `Role`"
                )))
            })?,
            username: user.username.clone(),
        };

        let refresh_token = self.generate_refresh_token(user).await?;
        let jwt = self.generate_jwt(&user_jwt, &refresh_token, None).await?;

        Ok(LoginReturn { jwt, refresh_token })
    }
}

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
