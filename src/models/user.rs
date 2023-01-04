use chrono::NaiveDateTime;
use entity::user::Model as UserModel;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;
use serde::{Deserialize, Serialize};

use crate::services::{UserService, UserServiceError};

use super::role::Role;

pub const ADMIN_USERNAME: &'static str = "admin";

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Role,
}

impl TryFrom<UserModel> for User {
    type Error = ();

    fn try_from(model: UserModel) -> Result<Self, Self::Error> {
        Ok(Self {
            role: model.role.try_into()?,
            created_at: model.created_at,
            email: model.email,
            id: model.id,
            password: model.password,
            updated_at: model.updated_at,
            username: model.username,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLogin {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReturnDto {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: Role,
}

///
/// Request guard that will read the JWT from cookies and inject the user into the function
pub struct AuthUser {
    pub user: User,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        use rocket::http::Status;
        let token = req.cookies().get("token").map(|cookie| cookie.value());
        if let None = token {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        let token = token.unwrap();
        let user_service = UserService::from_request(req).await.succeeded();
        if let None = user_service {
            return Outcome::Failure((Status::InternalServerError, ()));
        }
        let mut user_service = user_service.unwrap();

        match user_service.validate_token(token).await {
            Ok(user) => {
                let user = User::try_from(user);
                if let Err(_) = user {
                    return Outcome::Failure((Status::InternalServerError, ()));
                }
                return Outcome::Success(Self {
                    user: user.unwrap(),
                });
            }
            Err(e) => match e {
                UserServiceError::InvalidToken => {
                    return Outcome::Failure((Status::Unauthorized, ()))
                }
                _ => return Outcome::Failure((Status::InternalServerError, ())),
            },
        }
    }
}
