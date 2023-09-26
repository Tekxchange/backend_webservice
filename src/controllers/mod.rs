use rocket::{Build, Rocket};
mod auth_controller;
mod product_controller;
mod user_controller;
mod file_controller;

#[options("/<_..>")]
fn options() {

}

pub fn mount_routes(r: Rocket<Build>) -> Rocket<Build> {
    r.mount("/api/users", user_controller::routes())
        .mount("/api/products", product_controller::routes())
        .mount("/api/auth", auth_controller::routes())
        .mount("/api/files", file_controller::routes())
        .mount("/", routes![options])
}
