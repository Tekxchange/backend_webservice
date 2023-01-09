use rocket::{response::status::Created, serde::json::Json, Route};

use crate::{
    models::{product::ProductDetails, user::AuthUser},
    services::{ProductService, ProductServiceError},
};

#[post("/create", format = "json", data = "<product_create>")]
async fn create_product(
    mut product_service: ProductService,
    product_create: Json<ProductDetails>,
    auth_user: AuthUser,
) -> Result<Created<()>, ProductServiceError> {
    product_service
        .create_new_product(product_create.0, auth_user)
        .await?;

    Ok(Created::new(""))
}

pub fn routes() -> Vec<Route> {
    routes![create_product]
}
