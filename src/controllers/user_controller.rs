use crate::{
    models::{
        role::Role,
        user::{AuthUser, UserReturnDto},
    },
    services::{UserService, UserServiceError},
};
use rocket::{serde::json::Json, Route};

#[cfg(test)]
mod test;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct UsernameExistsDto {
    username: String,
}

#[tracing::instrument(level = "trace")]
#[post("/username_exists", format = "json", data = "<username>")]
async fn username_exists(
    user_service: UserService,
    username: Json<UsernameExistsDto>,
) -> Result<Json<bool>, UserServiceError> {
    let found = user_service.username_exists(&username.0.username).await?;

    Ok(Json(found))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct EmailExistsDto {
    email: String,
}

#[tracing::instrument(level = "trace")]
#[post("/email_exists", format = "json", data = "<email>")]
async fn email_exists(
    user_service: UserService,
    email: Json<EmailExistsDto>,
) -> Result<Json<bool>, UserServiceError> {
    let found = user_service.email_exists(&email.0.email).await?;

    Ok(Json(found))
}

#[tracing::instrument(level = "trace")]
#[get("/user/info")]
async fn get_user_info(
    auth_user: AuthUser,
    user_service: UserService,
) -> Result<Json<UserReturnDto>, UserServiceError> {
    let user = user_service
        .get_user_by_id(&auth_user.user.id)
        .await?
        .unwrap();
    let to_return = UserReturnDto {
        id: user.id,
        email: user.email,
        username: user.username,
        role: Role::try_from(user.role).unwrap(),
    };

    Ok(Json(to_return))
}

pub fn routes() -> Vec<Route> {
    routes![username_exists, email_exists, get_user_info]
}
