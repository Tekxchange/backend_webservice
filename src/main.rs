#[macro_use] extern crate rocket;
mod controllers;
mod db;
mod models;
mod schema;
mod services;

#[launch]
fn rocket() -> _ {
    controllers::mount_routes(rocket::build())
}
