use crate::{models::user::UserRegister, services::UserService};
use rocket::{
    response::status::{BadRequest, Created},
    serde::json::Json,
    Route,
};

#[post("/register", format = "json", data = "<user_register>")]
async fn register(
    mut user_service: UserService,
    user_register: Json<UserRegister>,
) -> Result<Created<()>, BadRequest<()>> {
    user_service
        .create_user(user_register.0)
        .await
        .map_err(|_| BadRequest(None))?;

    let created_response = Created::new("");

    Ok(created_response)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct UsernameExistsDto {
    username: String,
}

#[post("/username_exists", format = "json", data = "<username>")]
async fn username_exists(
    mut user_service: UserService,
    username: Json<UsernameExistsDto>,
) -> Result<Json<bool>, BadRequest<()>> {
    let found = user_service
        .username_exists(&username.0.username)
        .await
        .map_err(|_| BadRequest(None))?;

    Ok(Json(found))
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct EmailExistsDto {
    email: String,
}

#[post("/email_exists", format = "json", data = "<email>")]
async fn email_exists(
    mut user_service: UserService,
    email: Json<EmailExistsDto>,
) -> Result<Json<bool>, BadRequest<()>> {
    let found = user_service
        .email_exists(&email.0.email)
        .await
        .map_err(|_| BadRequest(None))?;

    Ok(Json(found))
}

pub fn routes() -> Vec<Route> {
    return routes![register, username_exists, email_exists];
}
