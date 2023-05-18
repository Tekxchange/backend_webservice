use rocket::{
    response::status::{Accepted, Created},
    serde::json::Json,
    Route,
};

use crate::{
    models::{
        product::{ProductDetails, ProductReturn},
        user::AuthUser,
    },
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

#[get("/product?<id>", format = "json")]
async fn get_product_by_id(
    mut product_service: ProductService,
    id: i64,
) -> Result<Json<ProductReturn>, ProductServiceError> {
    let found_product = product_service.get_product_by_id(id).await?;

    Ok(Json(found_product))
}

#[put("/product?<id>", format = "json", data = "<product>")]
async fn update_product_by_id(
    mut product_service: ProductService,
    id: i64,
    user: AuthUser,
    product: Json<ProductDetails>,
) -> Result<Accepted<()>, ProductServiceError> {
    product_service
        .update_product_by_id(id, product.0, user)
        .await?;
    Ok(Accepted(None))
}

#[delete("/product?<id>")]
async fn delete_product_by_id(
    mut product_service: ProductService,
    id: i64,
    user: AuthUser,
) -> Result<(), ProductServiceError> {
    product_service.delete_product_by_id(id, user).await?;

    Ok(())
}

pub fn routes() -> Vec<Route> {
    routes![
        create_product,
        get_product_by_id,
        update_product_by_id,
        delete_product_by_id
    ]
}
