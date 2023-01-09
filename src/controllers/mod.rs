use rocket::{Build, Rocket};
mod product_controller;
mod user_controller;

pub fn mount_routes(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/api/users", user_controller::routes())
        .mount("/api/products", product_controller::routes())
}
