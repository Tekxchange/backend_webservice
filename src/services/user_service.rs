use crate::{
    db::establish_connection,
    models::{
        role::Role,
        user::{UserLogin, UserRegister},
    },
};
use argon2::{self, Config};
use chrono::offset::Utc;
use entity::user::{Entity as UserEntity, Model as UserModel};
use hmac::{Hmac, Mac};
use jwt::{AlgorithmType, Header as JwtHeader};
use rand::{distributions::Alphanumeric, Rng};
use rocket::{
    http::Status,
    request::{self, FromRequest},
    response::Responder,
    Request, Response,
};
use sea_orm::{prelude::*, query::Condition, ActiveValue, DatabaseConnection, Set};
use serde_json::json;
use sha2::Sha512;
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
    #[error("User with that email, username, or id does not exist")]
    UserNotFound,
    #[error("Incorrect password provided")]
    InvalidPassword,
    #[error("An unknown error occurred")]
    Unknown,
}

impl<'r> Responder<'r, 'static> for UserServiceError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::DuplicateUserError | Self::InvalidPassword => {
                Response::build_from(json!({ "error": format!("{self}") }).respond_to(request)?)
                    .status(Status::BadRequest)
                    .ok()
            }
            Self::UserNotFound => {
                Response::build_from(json!({ "error": format!("{self}") }).respond_to(request)?)
                    .status(Status::NotFound)
                    .ok()
            }
            _ => Response::build().status(Status::InternalServerError).ok(),
        }
    }
}

pub struct UserService {
    db_connection: DatabaseConnection,
    jwt_key: Hmac<Sha512>,
    signing_alg: AlgorithmType,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserService {
    type Error = UserServiceError;

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let secret = env::var("ROCKET_SECRET_KEY").unwrap();

        let key = Hmac::new_from_slice(secret.as_bytes()).map_err(|_| UserServiceError::Unknown);
        if let Err(e) = key {
            return request::Outcome::Failure((rocket::http::Status::InternalServerError, e));
        }
        let key: Hmac<Sha512> = key.unwrap();

        match establish_connection()
            .await
            .map_err(|e| UserServiceError::DbError(e))
        {
            Ok(conn) => request::Outcome::Success(Self {
                db_connection: conn,
                jwt_key: key,
                signing_alg: AlgorithmType::Hs512,
            }),
            Err(e) => request::Outcome::Failure((rocket::http::Status::InternalServerError, e)),
        }
    }
}

impl UserService {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        let secret = env::var("ROCKET_SECRET_KEY").unwrap();

        let key = Hmac::new_from_slice(secret.as_bytes()).unwrap();
        Self {
            db_connection,
            jwt_key: key,
            signing_alg: AlgorithmType::Hs512,
        }
    }

    pub async fn create_user(
        &mut self,
        mut register: UserRegister,
    ) -> Result<i64, UserServiceError> {
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
                .map_err(|_| UserServiceError::Unknown)?;

        let user_active_model = user::ActiveModel {
            email: ActiveValue::Set(register.email.to_owned()),
            password: ActiveValue::Set(register.password.to_owned()),
            username: ActiveValue::Set(register.username.to_owned()),
            ..Default::default()
        };

        let created_user = user_active_model
            .insert(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;
        Ok(created_user.id)
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

    pub async fn get_user_by_id(
        &mut self,
        id: &i64,
    ) -> Result<Option<UserModel>, UserServiceError> {
        Ok(UserEntity::find_by_id(*id)
            .one(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?)
    }

    fn generate_jwt(&mut self, user: &UserModel) -> Result<String, UserServiceError> {
        use jwt::{SignWithKey, Token};

        let user_id = user.id.to_string();
        let role_id = user.role.to_string();
        let now = Utc::now().timestamp().to_string();

        let mut claims: BTreeMap<&str, &str> = BTreeMap::new();
        claims.insert("sub", &user_id);
        claims.insert("role", &role_id);
        claims.insert("iat", &now);

        let jwt_headers = JwtHeader {
            algorithm: self.signing_alg,
            ..Default::default()
        };

        let token = Token::new(jwt_headers, claims)
            .sign_with_key(&self.jwt_key)
            .map_err(|e| {
                println!("{e:?}");
                UserServiceError::Unknown
            })?;

        Ok(token.as_str().to_owned())
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

        match argon2::verify_encoded(&user.password, login.password.as_bytes()) {
            Ok(success) => {
                if !success {
                    return Err(UserServiceError::InvalidPassword);
                }
                return Ok(self.generate_jwt(&user)?);
            }
            Err(e) => {
                println!("{e:?}");
                todo!();
            }
        }
    }

    pub async fn update_role_for_user(
        &mut self,
        user_id: i64,
        new_role: Role,
    ) -> Result<(), UserServiceError> {
        let mut user: entity::user::ActiveModel = self
            .get_user_by_id(&user_id)
            .await?
            .ok_or(UserServiceError::UserNotFound)?
            .into();

        user.role = Set(new_role as i16);
        user.update(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::OrmError(e))?;

        Ok(())
    }
}
