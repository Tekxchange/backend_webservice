use crate::{
    db::establish_connection,
    models::user::{UserLogin, UserRegister},
};
use argon2::{self, Config};
use entity::user::{Entity as UserEntity, Model as UserModel};
use migration::Condition;
use rand::{distributions::Alphanumeric, Rng};
use rocket::request::{self, FromRequest};
use rocket::Request;
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::*, ActiveValue};
use std::{collections::BTreeMap, env};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("This username and/or email already exists")]
    DuplicateUserError,
    #[error(transparent)]
    DbError(crate::db::DbError),
    #[error(transparent)]
    OrmError(sea_orm::DbErr),
    #[error("User with that email or username does not exist")]
    UserNotFound,
    #[error("Incorrect password provided")]
    InvalidPassword,
    #[error("An unknown error occurred")]
    Unknown,
}

pub struct UserService {
    db_connection: DatabaseConnection,
    secret: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserService {
    type Error = UserServiceError;

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let secret = env::var("SECRET").unwrap();
        match establish_connection()
            .await
            .map_err(|e| UserServiceError::DbError(e))
        {
            Ok(conn) => request::Outcome::Success(Self {
                db_connection: conn,
                secret,
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

        let salt: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        let config = Config::default();

        register.password =
            argon2::hash_encoded(register.password.as_bytes(), salt.as_bytes(), &config)
                .map_err(|e| UserServiceError::Unknown)?;

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

    async fn get_by_email(&mut self, email: &str) -> Result<Option<UserModel>, UserServiceError> {
        use entity::user;
        let found = UserEntity::find()
            .filter(Condition::all().add(user::Column::Email.like(email)))
            .one(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;

        Ok(found)
    }

    async fn get_by_username(
        &mut self,
        username: &str,
    ) -> Result<Option<UserModel>, UserServiceError> {
        use entity::user;
        let found = UserEntity::find()
            .filter(Condition::all().add(user::Column::Username.like(username)))
            .one(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;

        Ok(found)
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

    pub async fn login(&mut self, login: UserLogin) -> Result<String, UserServiceError> {
        use hmac::{Hmac, Mac};
        use jwt::{Header, SignWithKey, Token};
        use sha2::Sha256;
        let mut user: Option<UserModel> = None;

        if let Some(ref email) = login.email {
            user = self.get_by_email(email).await?;
        } else if let Some(ref username) = login.username {
            user = self.get_by_username(username).await?;
        }

        if let None = user {
            return Err(UserServiceError::UserNotFound);
        }

        let user: UserModel = user.unwrap();

        let password_hash: PasswordHash =
            PasswordHash::new(&user.password).map_err(|_| UserServiceError::InvalidPassword)?;

        println!("{:?}", &password_hash);

        match Argon2::default().verify_password(&user.password.as_bytes(), &password_hash) {
            Ok(_) => {
                let key: Hmac<Sha256> = Hmac::new_from_slice(self.secret.as_bytes())
                    .map_err(|_| UserServiceError::Unknown)?;

                let header = Header {
                    ..Default::default()
                };

                let user_id_str = user.id.to_string();

                let mut claims: BTreeMap<&str, &str> = BTreeMap::new();
                claims.insert("sub", &user_id_str);

                let token = Token::new(header, claims)
                    .sign_with_key(&key)
                    .map_err(|_| UserServiceError::Unknown)?;

                Ok(token.as_str().to_owned())
            }
            Err(e) => {
                println!("{:?}", e);
                return Err(UserServiceError::InvalidPassword);
            }
        }
    }
}
