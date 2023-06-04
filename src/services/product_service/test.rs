use crate::models::{
    product::{ProductDetails, ProductReturn},
    role::Role,
    user::{AuthUser, UserJwtDto},
};
use crate::{
    db::test::establish_connection,
    models::user::UserRegister,
    services::{ProductService, UserService},
};
use entity::user::Model as UserModel;
use geolocation_utils::Coordinate;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use sea_orm::DatabaseConnection;

type E = Result<(), Box<dyn std::error::Error>>;

async fn create_test_user(db: DatabaseConnection, username: &str) -> UserModel {
    let us = UserService::new(db);
    let id = us
        .create_user(
            UserRegister {
                email: format!("{username}@test.com"),
                password: "testPass".into(),
                username: username.into(),
            },
            false,
        )
        .await
        .unwrap();
    let user = us.get_user_by_id(&id).await.unwrap().unwrap();

    user
}

async fn create_test_product(
    ps: &ProductService,
    user: UserModel,
    coords: Coordinate,
) -> ProductReturn {
    let id = ps
        .create_new_product(
            ProductDetails {
                description: "description".into(),
                title: "title".into(),
                price: Decimal::new(5, 15),
                country: "country".into(),
                state: "state".into(),
                city: "city".into(),
                zip: "zip".into(),
                latitude: Some(Decimal::from_f64(coords.latitude).unwrap()),
                longitude: Some(Decimal::from_f64(coords.longitude).unwrap()),
            },
            AuthUser {
                user: UserJwtDto {
                    id: user.id,
                    username: user.username,
                    role: Role::try_from(user.role).unwrap(),
                },
            },
        )
        .await
        .unwrap();

    ps.get_product_by_id(id).await.unwrap()
}

mod create_new_product {
    use rust_decimal::prelude::FromPrimitive;

    use super::*;

    #[tokio::test]
    async fn successful_creation() -> E {
        let db = establish_connection().await?;
        let u = create_test_user(db.clone(), "testUser").await;
        let ps = ProductService::new(db);
        let created = ps
            .create_new_product(
                ProductDetails {
                    city: "some city".into(),
                    country: "some country".into(),
                    description: "some description".into(),
                    state: "some state".into(),
                    title: "some title".into(),
                    zip: "some zip".into(),
                    latitude: Some(Decimal::new(0, 0)),
                    longitude: Some(Decimal::new(0, 0)),
                    price: Decimal::new(0, 15),
                },
                AuthUser {
                    user: UserJwtDto {
                        id: u.id,
                        role: Role::User,
                        username: u.username,
                    },
                },
            )
            .await;
        assert!(created.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn unsuccessful_creation_missing_user() -> E {
        let db = establish_connection().await?;
        let ps = ProductService::new(db);

        let res = ps
            .create_new_product(
                ProductDetails {
                    description: "test".into(),
                    title: "title".into(),
                    price: Decimal::new(0, 15),
                    country: "country".into(),
                    state: "state".into(),
                    city: "city".into(),
                    zip: "zip".into(),
                    latitude: Some(Decimal::from_f64(1.24).unwrap()),
                    longitude: Some(Decimal::from_f64(1.24).unwrap()),
                },
                AuthUser {
                    user: UserJwtDto {
                        id: 12,
                        username: "test".into(),
                        role: Role::User,
                    },
                },
            )
            .await;

        assert!(res.is_err());

        Ok(())
    }
}

mod search_for_products {
    use crate::dtos::product::ProductFilter;

    use super::*;

    #[tokio::test]
    async fn gets_products_at_location_and_radius() -> E {
        let db = establish_connection().await?;
        let user = create_test_user(db.clone(), "testUser").await;
        let product_service = ProductService::new(db);
        create_test_product(
            &product_service,
            user.clone(),
            Coordinate::new(1.0, 1.2500025),
        )
        .await;

        create_test_product(&product_service, user, Coordinate::new(1.0, 1.25)).await;

        let found = product_service
            .search_for_products(ProductFilter {
                city: None,
                query: None,
                zip: None,
                coordinate: Coordinate::new(1.0, 1.250003),
                price_high: None,
                price_low: None,
                product_id_lower: None,
                radius: Decimal::from_f64(1.0).unwrap(),
                units: None,
            })
            .await?;
        assert_eq!(found.len(), 2);

        Ok(())
    }
}
