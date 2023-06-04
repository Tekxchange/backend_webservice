use crate::{
    dtos::product::ProductFilter,
    models::{
        product::{ProductDetails, ProductLocationReturn, ProductReturn},
        user::{AuthUser, MinUserReturnDto},
    },
};
use entity::product::{self, ActiveModel as ProductActiveModel, Entity as ProductEntity};
use rocket::{
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest},
    response::Responder,
    Request, Response,
};
use rust_decimal::prelude::*;
use sea_orm::{
    entity::prelude::*, query::Condition, ActiveModelTrait, ActiveValue, DatabaseConnection,
    QueryOrder, QuerySelect,
};
use serde_json::json;
use thiserror::Error;
#[cfg(test)]
mod test;

#[derive(Error, Debug)]
pub enum ProductServiceError {
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
            Self::OrmError(_) | Self::Unknown => {
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

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        req.rocket()
            .state::<DatabaseConnection>()
            .map(|db| Self::new(db.clone()))
            .or_forward(())
    }
}

impl ProductService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db_connection: db }
    }

    pub async fn create_new_product(
        &self,
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
            location_zip: ActiveValue::Set(create.zip),
            ..Default::default()
        };

        let created = to_create
            .insert(&self.db_connection)
            .await
            .map_err(ProductServiceError::OrmError)?;

        Ok(created.id)
    }

    pub async fn get_product_by_id(&self, id: i64) -> Result<ProductReturn, ProductServiceError> {
        let found = ProductEntity::find_by_id(id)
            .find_also_related(entity::user::Entity)
            .one(&self.db_connection)
            .await
            .map_err(|e| ProductServiceError::OrmError(e))?;

        if let Some((prod, Some(user))) = found {
            let pics = entity::product_picture::Entity::find()
                .filter(entity::product_picture::Column::ProductId.eq(prod.id))
                .all(&self.db_connection)
                .await
                .map_err(|e| ProductServiceError::OrmError(e))?;

            Ok(ProductReturn {
                title: prod.product_title,
                description: prod.description,
                price: prod.price,
                created_by: MinUserReturnDto {
                    id: user.id,
                    username: user.username,
                },
                latitude: prod.location_latitude,
                longitude: prod.location_longitude,
                pictures: pics.into_iter().map(|i| i.id).collect(),
            })
        } else {
            Err(ProductServiceError::NotFound(id))
        }
    }

    pub async fn update_product_by_id(
        &self,
        id: i64,
        product: ProductDetails,
        user: AuthUser,
    ) -> Result<(), ProductServiceError> {
        let db_product = self.get_product_by_id(id).await?;
        if db_product.created_by.id != user.user.id {
            return Err(ProductServiceError::NotAllowed);
        }

        let active_product: entity::product::ActiveModel = ProductEntity::find()
            .filter(Condition::all().add(entity::product::Column::Id.eq(id)))
            .one(&self.db_connection)
            .await
            .map_err(ProductServiceError::OrmError)?
            .ok_or(ProductServiceError::NotFound(id))?
            .into();

        let new_prod = entity::product::ActiveModel {
            description: ActiveValue::Set(product.description),
            location_city: ActiveValue::Set(product.city),
            location_country: ActiveValue::Set(product.country),
            location_state: ActiveValue::Set(product.state),
            location_zip: ActiveValue::Set(product.zip),
            location_latitude: ActiveValue::Set(product.latitude),
            location_longitude: ActiveValue::Set(product.longitude),
            price: ActiveValue::Set(product.price),
            product_title: ActiveValue::Set(product.title),
            ..active_product
        };

        new_prod
            .update(&self.db_connection)
            .await
            .map_err(ProductServiceError::OrmError)?;

        Ok(())
    }

    pub async fn delete_product_by_id(
        &self,
        id: i64,
        user: AuthUser,
    ) -> Result<(), ProductServiceError> {
        let product: ProductActiveModel = ProductEntity::find_by_id(id)
            .one(&self.db_connection)
            .await
            .map_err(ProductServiceError::OrmError)?
            .ok_or(ProductServiceError::NotFound(id))?
            .into();

        if product.created_by.clone().unwrap() != user.user.id {
            return Err(ProductServiceError::NotAllowed);
        }

        product
            .delete(&self.db_connection)
            .await
            .map_err(ProductServiceError::OrmError)?;

        Ok(())
    }

    pub async fn search_for_products(
        &self,
        filter: ProductFilter,
    ) -> Result<Vec<ProductLocationReturn>, ProductServiceError> {
        let bounds = geolocation_utils::CoordinateBoundaries::new(
            filter.coordinate,
            filter.radius.to_f64().unwrap(),
            filter.units,
        )
        .ok_or_else(|| ProductServiceError::Unknown)?;

        let mut found = ProductEntity::find().filter(
            Condition::all()
                .add(product::Column::LocationLatitude.gte(bounds.min_latitude()))
                .add(product::Column::LocationLatitude.lte(bounds.max_latitude()))
                .add(product::Column::LocationLongitude.gte(bounds.min_longitude()))
                .add(product::Column::LocationLongitude.lte(bounds.max_longitude())),
        );

        if let Some(high) = filter.price_high {
            found = found.filter(product::Column::Price.lte(high));
        }
        if let Some(low) = filter.price_low {
            found = found.filter(product::Column::Price.gte(low));
        }
        if let Some(query) = filter.query {
            found = found.filter(product::Column::ProductTitle.like(&query));
        }
        if let Some(zip) = filter.zip {
            found = found.filter(product::Column::LocationZip.like(&zip));
        }
        if let Some(city) = filter.city {
            found = found.filter(product::Column::LocationCity.like(&city));
        }
        if let Some(id_lower) = filter.product_id_lower {
            found = found.filter(product::Column::Id.gte(id_lower));
        }

        let found = found
            .limit(25)
            .order_by(product::Column::Id, sea_orm::Order::Desc)
            .all(&self.db_connection)
            .await
            .map_err(ProductServiceError::OrmError)?;

        Ok(found
            .into_iter()
            .map(|prod| {
                Some(ProductLocationReturn {
                    latitude: prod.location_latitude.unwrap(),
                    longitude: prod.location_longitude.unwrap(),
                    id: prod.id,
                })
            })
            .filter(|item| item.is_some())
            .map(|item| item.unwrap())
            .collect())
    }
}
