use crate::{models::user::UserRegister, services::UserService};
use rocket::{response::status::Created, serde::json::Json, Route};

#[post("/register", format = "json", data = "<user_register>")]
fn register(
    mut user_service: UserService,
    user_register: Json<UserRegister>,
) -> Result<Created<()>, ()> {
    user_service.create_user(user_register.0).map_err(|_| ())?;

    let created_response = Created::new("");

    Ok(created_response)
}

pub fn routes() -> Vec<Route> {
    return routes![register];
}
