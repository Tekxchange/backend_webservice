use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use sea_orm::DatabaseConnection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthServiceError {
    #[error(transparent)]
    Unknown(anyhow::Error),
}

pub struct AuthService {
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn hash_password(password: &str) -> Result<String, AuthServiceError> {
        let salt = SaltString::generate(OsRng);
        let argon = Argon2::default();

        let hashed = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthServiceError::Unknown(anyhow::anyhow!(e)))?;

        Ok(hashed.to_string())
    }

    pub fn verify_password(
        encoded_password: &str,
        raw_password: &str,
    ) -> Result<bool, AuthServiceError> {
        let argon = Argon2::default();
        let hash = PasswordHash::new(encoded_password)
            .map_err(|e| AuthServiceError::Unknown(anyhow::anyhow!(e)))?;

        match argon.verify_password(raw_password.as_bytes(), &hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
