use rocket::{Build, Rocket};
mod user_controller;

pub fn mount_routes(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/api/users", user_controller::routes())
}
