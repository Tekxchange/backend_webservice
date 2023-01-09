use entity::product::ActiveModel as ProductActiveModel;
use rocket::{
    http::Status,
    outcome::Outcome,
    request::{self, FromRequest},
    response::Responder,
    Request, Response,
};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
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
}

impl<'r> Responder<'r, 'static> for ProductServiceError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::DbError(_) | Self::OrmError(_) => {
                Response::build().status(Status::InternalServerError).ok()
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
            ..Default::default()
        };

        let created = to_create
            .insert(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        Ok(created.id)
    }
}
