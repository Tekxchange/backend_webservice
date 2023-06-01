use crate::models::{
    product::ProductDetails,
    role::Role,
    user::{AuthUser, UserJwtDto},
};
use crate::{
    db::test::establish_connection,
    models::user::UserRegister,
    services::{ProductService, UserService},
};
use entity::user::Model as UserModel;
use rust_decimal::Decimal;
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

async fn create_test_product(ps: &ProductService, user: UserModel) -> i64 {
    ps.create_new_product(
        ProductDetails {
            description: "description".into(),
            title: "title".into(),
            price: Decimal::new(5, 15),
            country: "country".into(),
            state: "state".into(),
            city: "city".into(),
            zip: "zip".into(),
            latitude: Some(Decimal::new(13, 0)),
            longitude: Some(Decimal::new(12, 0)),
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
    .unwrap()
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
