use rocket::{response::status::Created, serde::json::Json, Route};

use crate::{
    dtos::auth::LoginReturn,
    models::user::{RefreshAuthUser, UserLogin, UserRegister},
    services::{AuthService, AuthServiceError, UserService, UserServiceError},
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
    auth_service: AuthService,
    login: Json<UserLogin>,
) -> Result<Json<LoginReturn>, UserServiceError> {
    let token = user_service.login(login.0, auth_service).await?;

    Ok(Json(token))
}

#[get("/refresh")]
async fn refresh_login(
    mut auth_service: AuthService,
    auth_user: RefreshAuthUser,
) -> Result<Json<String>, AuthServiceError> {
    let jwt = auth_service
        .generate_jwt(&auth_user.user, &auth_user.refresh_token)
        .await?;

    Ok(Json(jwt))
}

#[post("/revoke_token")]
async fn revoke_refresh_token(
    mut auth_service: AuthService,
    user_service: UserService,
) -> Result<(), AuthServiceError> {
    let first = user_service.get_user_by_id(&1).await.unwrap().unwrap();

    auth_service.revoke_refresh_token(&first).await?;
    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![register, login, refresh_login, revoke_refresh_token]
}
