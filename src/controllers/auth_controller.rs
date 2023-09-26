use crate::{
    models::user::{AuthUser, RefreshAuthUser, UserLogin, UserRegister},
    services::{AuthService, AuthServiceError, UserService, UserServiceError},
};
use rocket::{
    http::{Cookie, CookieJar},
    response::status::Created,
    serde::json::Json,
    time::{Duration, OffsetDateTime},
    Route,
};
use serde::Serialize;

#[tracing::instrument(level = "trace")]
#[post("/register", format = "json", data = "<user_register>")]
async fn register(
    user_service: UserService,
    user_register: Json<UserRegister>,
) -> Result<Created<()>, UserServiceError> {
    user_service.create_user(user_register.0, false).await?;

    let created_response = Created::new("");

    Ok(created_response)
}

#[derive(Serialize)]
pub struct JwtReturn {
    jwt: String,
}

#[tracing::instrument(level = "error")]
#[post("/login", format = "json", data = "<login>")]
async fn login(
    user_service: UserService,
    auth_service: AuthService,
    login: Json<UserLogin>,
    cookies: &CookieJar<'_>,
) -> Result<Json<JwtReturn>, UserServiceError> {
    let token = user_service.login(login.0, auth_service).await?;

    let mut refresh_expires = OffsetDateTime::now_utc();
    refresh_expires += Duration::days(30);

    let refresh_cookie = Cookie::build("refresh", token.refresh_token)
        .http_only(true)
        .expires(refresh_expires)
        .same_site(rocket::http::SameSite::Strict)
        .secure(true)
        .finish();

    cookies.add(refresh_cookie);

    Ok(Json(JwtReturn { jwt: token.jwt }))
}

#[tracing::instrument(level = "trace")]
#[get("/refresh")]
async fn refresh_login(
    mut auth_service: AuthService,
    auth_user: RefreshAuthUser,
    cookies: &CookieJar<'_>,
) -> Result<Json<JwtReturn>, AuthServiceError> {
    let jwt = auth_service
        .generate_jwt(&auth_user.user, &auth_user.refresh_token, None)
        .await?;

    let mut refresh_expires = OffsetDateTime::now_utc();
    refresh_expires += Duration::days(30);

    let refresh_cookie = Cookie::build("refresh", auth_user.refresh_token.to_owned())
        .http_only(true)
        .expires(refresh_expires)
        .same_site(rocket::http::SameSite::Strict)
        .secure(true)
        .finish();

    cookies.add(refresh_cookie);

    Ok(Json(JwtReturn { jwt }))
}

#[tracing::instrument(level = "trace")]
#[get("/revoke_token")]
async fn revoke_refresh_token(
    mut auth_service: AuthService,
    user_service: UserService,
    auth_user: AuthUser,
    cookies: &CookieJar<'_>
) -> Result<(), AuthServiceError> {
    let first = user_service
        .get_user_by_id(&auth_user.user.id)
        .await
        .unwrap()
        .unwrap();

    auth_service.revoke_refresh_token(&first).await?;
    
    cookies.remove(Cookie::named("refresh"));

    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![register, login, refresh_login, revoke_refresh_token]
}
