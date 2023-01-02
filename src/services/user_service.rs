use crate::{db::establish_connection, models::user::UserRegister};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use entity::user::Entity as UserEntity;
use migration::Condition;
use rocket::request::{self, FromRequest};
use rocket::Request;
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::*, ActiveValue};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("This username and/or email already exists")]
    DuplicateUserError,
    #[error(transparent)]
    DbError(crate::db::DbError),
    #[error(transparent)]
    OrmError(sea_orm::DbErr),
    #[error("An unknown error occurred")]
    Unknown,
}

pub struct UserService {
    db_connection: DatabaseConnection,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserService {
    type Error = UserServiceError;

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match establish_connection()
            .await
            .map_err(|e| UserServiceError::DbError(e))
        {
            Ok(conn) => request::Outcome::Success(Self {
                db_connection: conn,
            }),
            Err(e) => request::Outcome::Failure((rocket::http::Status::InternalServerError, e)),
        }
    }
}

impl UserService {
    pub async fn create_user(
        &mut self,
        mut register: UserRegister,
    ) -> Result<(), UserServiceError> {
        use entity::user;

        let found_users = UserEntity::find()
            .filter(
                Condition::any()
                    .add(user::Column::Username.like(&register.username))
                    .add(user::Column::Email.like(&register.email)),
            )
            .count(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;

        if found_users > 0 {
            return Err(UserServiceError::DuplicateUserError);
        }

        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(register.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        register.password = password_hash;

        let user_active_model = user::ActiveModel {
            email: ActiveValue::Set(register.email.to_owned()),
            password: ActiveValue::Set(register.password.to_owned()),
            username: ActiveValue::Set(register.username.to_owned()),
            ..Default::default()
        };

        user_active_model
            .insert(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;
        Ok(())
    }

    pub async fn username_exists(&mut self, username: &str) -> Result<bool, UserServiceError> {
        use entity::user;

        let found_count = UserEntity::find()
            .filter(Condition::any().add(user::Column::Username.like(username)))
            .count(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;

        return Ok(found_count > 0);
    }

    pub async fn email_exists(&mut self, email: &str) -> Result<bool, UserServiceError> {
        use entity::user;

        let found_count = UserEntity::find()
            .filter(Condition::any().add(user::Column::Email.like(email)))
            .count(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;

        return Ok(found_count > 0);
    }
}
