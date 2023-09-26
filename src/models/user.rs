use super::role::Role;
use crate::services::{AuthService, UserService};
use chrono::NaiveDateTime;
use entity::user::Model as UserModel;
use rocket::{
    http::Status,
    outcome::{try_outcome, Outcome},
    request::{self, FromRequest},
    Request,
};
use serde::{Deserialize, Serialize};

pub const ADMIN_USERNAME: &str = "admin";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthServiceModel {
    pub id: i64,
    pub username: String,
    pub role_id: i16,
}

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserJwtDto {
    pub id: i64,
    pub username: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinUserReturnDto {
    pub id: i64,
    pub username: String,
}

/// Request guard that will read the JWT from headers and inject the user into the function
#[derive(Debug)]
pub struct AuthUser {
    pub user: UserJwtDto,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_service: AuthService = try_outcome!(req.guard::<AuthService>().await);

        let jwt = match req.headers().get("authorization").next() {
            Some(j) => j,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        let user = match auth_service.validate_jwt(jwt.into(), None) {
            Ok(user) => user,
            Err(_) => return Outcome::Failure((Status::Unauthorized, ())),
        };

        Outcome::Success(AuthUser { user })
    }
}

/// Request guard that will read the JWT from headers and inject the user into the function
/// ## Important
/// - Only to be used when refreshing jwt
#[derive(Debug)]
pub struct RefreshAuthUser {
    pub user: UserJwtDto,
    pub refresh_token: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RefreshAuthUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let mut auth_service: AuthService = try_outcome!(req.guard::<AuthService>().await);
        let user_service: UserService = try_outcome!(req.guard::<UserService>().await);

        let refresh = match req.cookies().get("refresh") {
            Some(r) => r.value(),
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        let user_id = match match auth_service.get_user_from_refresh(refresh.to_owned()).await {
            Ok(id) => id,
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        } {
            Some(id) => id,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        let user = match match user_service.get_user_by_id(&user_id).await {
            Ok(model) => model,
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        } {
            Some(model) => model,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        Outcome::Success(RefreshAuthUser {
            user: UserJwtDto {
                id: user.id,
                username: user.username,
                role: Role::try_from(user.role).unwrap(),
            },
            refresh_token: refresh.to_owned(),
        })
    }
}
