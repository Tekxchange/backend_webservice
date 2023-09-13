use crate::{
    dtos::product::{ProductCreated, ProductFilter},
    models::{
        product::{ProductDetails, ProductLocationReturn, ProductReturn, ProductReturnNoUser},
        user::AuthUser,
    },
    services::{ProductService, ProductServiceError},
};
use rocket::{
    response::status::{Accepted, Created},
    serde::json::Json,
    Route,
};

#[post("/create", format = "json", data = "<product_create>")]
async fn create_product(
    product_service: ProductService,
    product_create: Json<ProductDetails>,
    auth_user: AuthUser,
) -> Result<Created<Json<ProductCreated>>, ProductServiceError> {
    let id = product_service
        .create_new_product(product_create.0, auth_user)
        .await?;

    let created =
        Created::new(format!("/api/products/product?id={id}")).body(Json(ProductCreated { id }));

    Ok(created)
}

#[get("/product?<id>")]
async fn get_product_by_id(
    product_service: ProductService,
    id: i64,
) -> Result<Json<ProductReturn>, ProductServiceError> {
    let found_product = product_service.get_product_by_id(id).await?;

    Ok(Json(found_product))
}

#[put("/product?<id>", format = "json", data = "<product>")]
async fn update_product_by_id(
    product_service: ProductService,
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
    product_service: ProductService,
    id: i64,
    user: AuthUser,
) -> Result<(), ProductServiceError> {
    product_service.delete_product_by_id(id, user).await?;

    Ok(())
}

#[post("/search", data = "<filter>")]
async fn search_for_products(
    filter: Json<ProductFilter>,
    product_service: ProductService,
) -> Result<Json<Vec<ProductLocationReturn>>, ProductServiceError> {
    let found_products = product_service.search_for_products(filter.0).await?;

    Ok(Json(found_products))
}

#[get("/by_user?<user_id>&<limit>&<lower_limit>")]
async fn get_products_by_user_id(
    product_service: ProductService,
    user_id: i64,
    limit: Option<u64>,
    lower_limit: Option<i64>,
) -> Result<Json<Vec<ProductReturnNoUser>>, ProductServiceError> {
    let to_return = product_service
        .get_products_by_user_id(user_id, limit, lower_limit)
        .await?;

    Ok(Json(to_return))
}

pub fn routes() -> Vec<Route> {
    routes![
        create_product,
        get_product_by_id,
        update_product_by_id,
        delete_product_by_id,
        search_for_products,
        get_products_by_user_id
    ]
}
