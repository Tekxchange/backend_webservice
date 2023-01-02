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

pub fn routes() -> Vec<Route> {
    return routes![register];
}
