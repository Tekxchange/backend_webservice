use entity::product::{ActiveModel as ProductActiveModel, Entity as ProductEntity};
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    response::Responder,
    Request, Response,
};
use sea_orm::prelude::*;
use sea_orm::query::Condition;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use serde_json::json;
use thiserror::Error;

use crate::{
    db::establish_connection,
    models::{product::ProductDetails, user::AuthUser},
};

#[derive(Error, Debug)]
pub enum ProductServiceError {
    #[error(transparent)]
    DbError(crate::db::DbError),
    #[error(transparent)]
    OrmError(sea_orm::DbErr),
    #[error("Product with id {0} not found")]
    NotFound(i64),
    #[error("You are not authorized to perform changes on this product")]
    NotAllowed,
    #[error("An unknown error occurred")]
    Unknown,
}

impl<'r> Responder<'r, 'static> for ProductServiceError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::DbError(_) | Self::OrmError(_) | Self::Unknown => {
                Response::build().status(Status::InternalServerError).ok()
            }
            Self::NotFound(_) => {
                Response::build_from(json!({ "error": format!("{self}") }).respond_to(request)?)
                    .status(Status::NotFound)
                    .ok()
            }
            Self::NotAllowed => {
                Response::build_from(json!({ "error": format!("{self}") }).respond_to(request)?)
                    .status(Status::Forbidden)
                    .ok()
            }
        }
    }
}

pub struct ProductService {
    db_connection: DatabaseConnection,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ProductService {
    type Error = ProductServiceError;

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match establish_connection()
            .await
            .map_err(|e| ProductServiceError::DbError(e))
        {
            Err(e) => return Outcome::Failure((Status::InternalServerError, e)),
            Ok(db) => return Outcome::Success(Self { db_connection: db }),
        }
    }
}

impl ProductService {
    pub async fn create_new_product(
        &mut self,
        create: ProductDetails,
        creating_user: AuthUser,
    ) -> Result<i64, ProductServiceError> {
        let to_create = ProductActiveModel {
            price: ActiveValue::Set(create.price),
            description: ActiveValue::Set(create.description),
            product_title: ActiveValue::Set(create.title),
            created_by: ActiveValue::Set(creating_user.user.id),
            location_city: ActiveValue::Set(create.city),
            location_state: ActiveValue::Set(create.state),
            location_country: ActiveValue::Set(create.country),
            location_latitude: ActiveValue::Set(create.latitude),
            location_longitude: ActiveValue::Set(create.longitude),
            ..Default::default()
        };

        let created = to_create
            .insert(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        Ok(created.id)
    }

    pub async fn get_product_by_id(
        &mut self,
        id: i64,
    ) -> Result<ProductDetails, ProductServiceError> {
        use entity::product;
        let found = ProductEntity::find()
            .filter(Condition::any().add(product::Column::Id.eq(id)))
            .one(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        if let Some(product) = found {
            return Ok(ProductDetails {
                description: product.description,
                title: product.product_title,
                price: product.price,
                city: product.location_city,
                country: product.location_country,
                state: product.location_state,
                latitude: product.location_latitude,
                longitude: product.location_longitude,
            });
        }

        return Err(ProductServiceError::NotFound(id));
    }
}
