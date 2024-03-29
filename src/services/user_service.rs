use super::auth_service::AuthServiceError;
use crate::{
    dtos::auth::LoginReturn,
    models::{
        role::Role,
        user::{UserLogin, UserRegister},
    },
    services::auth_service::AuthService,
    AnyhowResponder,
};
use anyhow::anyhow;
use entity::user::{ActiveModel as UserActiveModel, Entity as UserEntity, Model as UserModel};
use lazy_static::lazy_static;
use regex::Regex;
use rocket::{
    outcome::IntoOutcome,
    request::{self, FromRequest},
    response::Responder,
    Request,
};
use sea_orm::{prelude::*, query::Condition, ActiveValue, DatabaseConnection, Set};
use thiserror::Error;
#[cfg(test)]
mod test;

lazy_static! {
    static ref INVALID_USERNAME_REGEX: Regex =
        Regex::new(r"(?i)(admin|moderator|fuck|ass|shit|cunt|piss|wank|twat)").unwrap();
}

#[derive(Error, Debug, Responder)]
pub enum UserServiceError {
    #[error("This username and/or email already exists")]
    #[response(status = 400)]
    DuplicateUserError(AnyhowResponder),
    #[error("An unknown error has occurred")]
    #[response(status = 500)]
    InternalError(AnyhowResponder),
    #[error("User with that email, username, or id does not exist")]
    #[response(status = 404)]
    UserNotFound(AnyhowResponder),
    #[error("The request contains forbidden words")]
    #[response(status = 400)]
    ForbiddenWords(AnyhowResponder),
    #[error(transparent)]
    AuthServiceError(AuthServiceError),
}

#[derive(Debug)]
pub struct UserService {
    db_connection: DatabaseConnection,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserService {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        req.rocket()
            .state::<DatabaseConnection>()
            .map(|conn| Self {
                db_connection: conn.clone(),
            })
            .or_forward(())
    }
}

impl UserService {
    pub fn new(db_connection: DatabaseConnection) -> Self {
        Self { db_connection }
    }

    pub async fn create_user(
        &self,
        mut register: UserRegister,
        bypass_name_check: bool,
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
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        if found_users > 0 {
            return Err(UserServiceError::DuplicateUserError(AnyhowResponder(
                anyhow!("User already exists"),
            )));
        }

        if !bypass_name_check && INVALID_USERNAME_REGEX.is_match(&register.username) {
            return Err(UserServiceError::ForbiddenWords(AnyhowResponder(anyhow!(
                "User attempted signup with a forbidden word in username"
            ))));
        }

        register.password = AuthService::hash_password(&register.password)
            .map_err(UserServiceError::AuthServiceError)?;

        let UserModel { id, .. } = UserActiveModel {
            email: ActiveValue::Set(register.email),
            password: ActiveValue::Set(register.password),
            username: ActiveValue::Set(register.username),
            ..Default::default()
        }
        .insert(&self.db_connection)
        .await
        .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(id)
    }

    async fn get_by_email(&self, email: &str) -> Result<Option<UserModel>, UserServiceError> {
        use entity::user;
        let found = UserEntity::find()
            .filter(Condition::all().add(user::Column::Email.like(email)))
            .one(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(found)
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<UserModel>, UserServiceError> {
        use entity::user;
        let found = UserEntity::find()
            .filter(Condition::all().add(user::Column::Username.like(username)))
            .one(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(found)
    }

    pub async fn get_user_by_id(&self, id: &i64) -> Result<Option<UserModel>, UserServiceError> {
        UserEntity::find_by_id(*id)
            .one(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))
    }

    pub async fn username_exists(&self, username: &str) -> Result<bool, UserServiceError> {
        use entity::user;

        let found_count = UserEntity::find()
            .filter(Condition::any().add(user::Column::Username.like(username)))
            .count(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(found_count > 0)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool, UserServiceError> {
        use entity::user;

        let found_count = UserEntity::find()
            .filter(Condition::any().add(user::Column::Email.like(email)))
            .count(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(found_count > 0)
    }

    pub async fn login(
        &self,
        login: UserLogin,
        mut auth_service: AuthService,
    ) -> Result<LoginReturn, UserServiceError> {
        let mut user: Option<UserModel> = None;

        if let Some(ref email) = login.email {
            user = self.get_by_email(email).await?;
        } else if let Some(ref username) = login.username {
            user = self.get_by_username(username).await?;
        }

        if user.is_none() {
            return Err(UserServiceError::UserNotFound(AnyhowResponder(anyhow!(
                "Requested user is not found"
            ))));
        }

        let user: UserModel = user.unwrap();

        let login_return = auth_service
            .login(login.password, &user)
            .await
            .map_err(UserServiceError::AuthServiceError)?;

        Ok(login_return)
    }

    pub async fn update_role_for_user(
        &self,
        user_id: i64,
        new_role: Role,
    ) -> Result<(), UserServiceError> {
        let mut user: entity::user::ActiveModel = self
            .get_user_by_id(&user_id)
            .await?
            .ok_or(UserServiceError::UserNotFound(AnyhowResponder(anyhow!(
                "Requested user is not found"
            ))))?
            .into();

        user.role = Set(new_role as i16);
        user.update(&self.db_connection)
            .await
            .map_err(|e| UserServiceError::InternalError(AnyhowResponder(anyhow!(e))))?;

        Ok(())
    }
}
