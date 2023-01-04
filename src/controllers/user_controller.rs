use crate::{
    models::user::{AuthUser, UserLogin, UserRegister, UserReturnDto},
    services::UserService,
};
use rocket::{
    http::{Cookie, CookieJar, SameSite},
    response::status::{BadRequest, Created, Unauthorized},
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

#[post("/login", format = "json", data = "<login>")]
async fn login(
    mut user_service: UserService,
    login: Json<UserLogin>,
    cookies: &CookieJar<'_>,
) -> Result<(), Unauthorized<()>> {
    let token = user_service.login(login.0).await.map_err(|e| {
        println!("{:?}", e);
        return Unauthorized(None);
    })?;

    let token_cookie = Cookie::build("token", token)
        .same_site(SameSite::Lax)
        .finish();

    cookies.add(token_cookie);

    Ok(())
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) {
    let token_cookie = cookies.get_pending("token");

    if let Some(cookie) = token_cookie {
        cookies.remove(cookie);
    }
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
    return routes![
        register,
        username_exists,
        email_exists,
        login,
        logout,
        get_user_info
    ];
}
