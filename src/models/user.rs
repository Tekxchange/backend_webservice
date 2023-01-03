use chrono::{DateTime, Utc};
use entity::user::Model as UserModel;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;
use serde::{Deserialize, Serialize};

use crate::services::{UserService, UserServiceError};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub username: String,
    pub email: String,
    pub password: String,
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

///
/// Request guard that will read the JWT from cookies and inject the user into the function
pub struct AuthUser {
    pub user: UserModel,
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
            Ok(user) => return Outcome::Success(Self { user }),
            Err(e) => match e {
                UserServiceError::InvalidToken => {
                    return Outcome::Failure((Status::Unauthorized, ()))
                }
                _ => return Outcome::Failure((Status::InternalServerError, ())),
            },
        }
    }
}
