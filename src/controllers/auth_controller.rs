use rocket::{
    http::{Cookie, CookieJar, SameSite},
    response::status::Created,
    serde::json::Json,
    Route,
};

use crate::{
    models::user::{UserLogin, UserRegister},
    services::{UserService, UserServiceError},
};

#[post("/register", format = "json", data = "<user_register>")]
async fn register(
    user_service: UserService,
    user_register: Json<UserRegister>,
) -> Result<Created<()>, UserServiceError> {
    user_service.create_user(user_register.0, false).await?;

    let created_response = Created::new("");

    Ok(created_response)
}

#[post("/login", format = "json", data = "<login>")]
async fn login(
    user_service: UserService,
    login: Json<UserLogin>,
    cookies: &CookieJar<'_>,
) -> Result<(), UserServiceError> {
    let token = user_service.login(login.0).await?;

    let token_cookie = Cookie::build("token", token)
        .same_site(SameSite::Lax)
        .finish();

    cookies.add(token_cookie);

    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![register, login]
}
