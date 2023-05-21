use rocket::Route;

#[post("/register")]
fn register() -> Result<(), ()> {
    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![register]
}
