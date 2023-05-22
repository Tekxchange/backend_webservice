use crate::{
    models::user::{AuthUser, UserReturnDto},
    services::{UserService, UserServiceError},
};
use rocket::{serde::json::Json, Route};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct UsernameExistsDto {
    username: String,
}

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

#[post("/email_exists", format = "json", data = "<email>")]
async fn email_exists(
    user_service: UserService,
    email: Json<EmailExistsDto>,
) -> Result<Json<bool>, UserServiceError> {
    let found = user_service.email_exists(&email.0.email).await?;

    Ok(Json(found))
}

#[get("/user/info")]
async fn get_user_info(auth_user: AuthUser) -> Json<UserReturnDto> {
    let to_return = UserReturnDto {
        id: auth_user.user.id,
        email: auth_user.user.email,
        username: auth_user.user.username,
        role: auth_user.user.role,
    };

    Json(to_return)
}

pub fn routes() -> Vec<Route> {
    routes![username_exists, email_exists, get_user_info]
}
