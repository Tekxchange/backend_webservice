use crate::{db::test::establish_connection, models::user::UserRegister, services::UserService};
use sea_orm::prelude::*;

use super::*;

type E = Result<(), Box<dyn std::error::Error>>;

async fn get_test_user(db: DatabaseConnection) -> entity::user::Model {
    let user_service = UserService::new(db);

    let created_id = user_service
        .create_user(
            UserRegister {
                username: String::from("JoeDiertay"),
                email: String::from("my@email.com"),
                password: String::from("my_p@ssw0rd"),
            },
            false,
        )
        .await
        .unwrap();
    let user = user_service
        .get_user_by_id(&created_id)
        .await
        .unwrap()
        .unwrap();

    user
}

#[test]
fn get_key_pair() {
    let kp = AuthService::get_key_pair();

    assert!(kp.is_ok());
}

#[test]
fn hash_password() -> E {
    let original = String::from("test password");

    let res = AuthService::hash_password(&original)?;

    assert_ne!(res, original);
    Ok(())
}

#[test]
fn verify_password() -> E {
    let original = String::from("test password");

    let hashed = AuthService::hash_password(&original)?;

    let validated = AuthService::verify_password(&hashed, &original)?;

    assert!(validated);
    Ok(())
}

mod generate_refresh_token {
    use super::*;

    #[tokio::test]
    async fn existing_redis() -> E {
        let db = establish_connection().await?;

        let test_user = get_test_user(db.clone()).await;

        let key = AuthService::get_key_pair()?;
        let mut redis = MockRedisRefresh::default();
        redis
            .expect_get_item()
            .times(1)
            .returning(|_| Ok(Some(String::from("returned_item"))));

        let mut auth_service = AuthService::new(db, Box::new(redis), key);

        let refresh = auth_service.generate_refresh_token(&test_user).await;
        assert!(refresh.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn missing_redis() -> E {
        let mut redis = MockRedisRefresh::default();
        redis.expect_get_item().times(1).returning(|_| Ok(None));
        redis.expect_set_item().times(1).returning(|_, _| Ok(()));

        let db = establish_connection().await?;
        let key = AuthService::get_key_pair()?;

        let test_user = get_test_user(db.clone()).await;

        let mut auth_service = AuthService::new(db, Box::new(redis), key);
        let token = auth_service.generate_refresh_token(&test_user).await;

        assert!(token.is_ok());
        Ok(())
    }
}

mod validate_refresh_token {
    use super::*;

    #[tokio::test]
    async fn has_redis_entry() -> E {
        let db = establish_connection().await?;
        let test_user = get_test_user(db.clone()).await;
        let mut redis = MockRedisRefresh::default();

        redis
            .expect_get_item()
            .returning(|_| Ok(Some(String::from("refresh_token"))))
            .times(1);

        let mut auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);

        let res = auth_service.validate_refresh_token(test_user.id).await?;

        assert!(res.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn has_no_redis_entry() -> E {
        let db = establish_connection().await?;
        let db_token = String::from("refresh_token");

        let test_user = get_test_user(db.clone()).await;
        entity::refresh_token::ActiveModel {
            token: ActiveValue::Set(db_token.to_owned()),
            user_id: ActiveValue::Set(test_user.id),
            ..Default::default()
        }
        .insert(&db)
        .await?;
        let mut redis = MockRedisRefresh::default();

        redis.expect_get_item().returning(|_| Ok(None)).times(1);
        redis
            .expect_set_item()
            .returning(move |_, value| {
                assert_eq!(&db_token, value);
                Ok(())
            })
            .times(1);

        let mut auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);

        let res = auth_service.validate_refresh_token(test_user.id).await?;

        assert!(res.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn has_no_entry() -> E {
        let db = establish_connection().await?;
        let mut redis = MockRedisRefresh::default();
        let key = AuthService::get_key_pair()?;
        let test_user = get_test_user(db.clone()).await;

        redis.expect_get_item().returning(|_| Ok(None)).times(1);

        let mut auth_service = AuthService::new(db, Box::new(redis), key);
        let found = auth_service.validate_refresh_token(test_user.id).await?;

        assert!(found.is_none());

        Ok(())
    }
}

mod generate_jwt {
    use super::*;

    #[tokio::test]
    async fn successful_generation() -> E {
        let refresh_token = String::from("refresh");
        let db = establish_connection().await?;
        let mut redis = MockRedisRefresh::default();

        let s = refresh_token.to_owned();
        redis
            .expect_get_item()
            .returning(move |_| Ok(Some(s.clone())))
            .times(1);

        let key = AuthService::get_key_pair()?;
        let test_user = get_test_user(db.clone()).await;

        let mut auth_service = AuthService::new(db, Box::new(redis), key);
        let jwt = auth_service
            .generate_jwt(
                &UserJwtDto {
                    id: test_user.id,
                    username: test_user.username,
                    role: Role::User,
                },
                &refresh_token,
                None,
            )
            .await;
        assert!(jwt.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn unsuccessful_generation_no_refresh_token() -> E {
        let db = establish_connection().await?;
        let mut redis = MockRedisRefresh::default();
        let user = get_test_user(db.clone()).await;

        redis.expect_get_item().returning(|_| Ok(None)).times(1);

        let mut auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);
        let found = auth_service
            .generate_jwt(
                &UserJwtDto {
                    id: user.id,
                    username: user.username,
                    role: Role::User,
                },
                "refresh",
                None,
            )
            .await;

        assert!(found.is_err());

        Ok(())
    }
}

mod validate_jwt {
    use super::*;

    #[tokio::test]
    async fn successful_not_expired() -> E {
        let refresh = String::from("refresh");
        let mut redis = MockRedisRefresh::default();
        let db = establish_connection().await?;
        let s = refresh.to_owned();
        redis
            .expect_get_item()
            .returning(move |_| Ok(Some(s.to_owned())))
            .times(1);
        let user = get_test_user(db.clone()).await;

        let mut auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);
        let token = auth_service
            .generate_jwt(
                &UserJwtDto {
                    id: user.id,
                    username: user.username,
                    role: Role::User,
                },
                &refresh,
                None,
            )
            .await?;

        let res = auth_service.validate_jwt(token, None);
        assert!(res.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn unsuccessful_expired() -> E {
        let refresh = String::from("refresh");
        let mut redis = MockRedisRefresh::default();
        let db = establish_connection().await?;
        let s = refresh.to_owned();
        redis
            .expect_get_item()
            .returning(move |_| Ok(Some(s.to_owned())))
            .times(1);
        let user = get_test_user(db.clone()).await;

        let mut auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);
        let token = auth_service
            .generate_jwt(
                &UserJwtDto {
                    id: user.id,
                    username: user.username,
                    role: Role::User,
                },
                &refresh,
                Some(Duration::from_secs(0)),
            )
            .await?;

        let res = auth_service.validate_jwt(token, None);

        assert!(res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn successful_with_tolerance() -> E {
        let refresh = String::from("refresh");
        let mut redis = MockRedisRefresh::default();
        let db = establish_connection().await?;
        let s = refresh.to_owned();
        redis
            .expect_get_item()
            .returning(move |_| Ok(Some(s.to_owned())))
            .times(1);
        let user = get_test_user(db.clone()).await;

        let mut auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);
        let token = auth_service
            .generate_jwt(
                &UserJwtDto {
                    id: user.id,
                    username: user.username,
                    role: Role::User,
                },
                &refresh,
                Some(Duration::from_secs(0)),
            )
            .await?;

        let res = auth_service.validate_jwt(token, Some(Duration::from_secs(10)));

        assert!(res.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn unsuccessful_invalid() -> E {
        let redis = MockRedisRefresh::default();
        let db = establish_connection().await?;
        let auth_service = AuthService::new(db, Box::new(redis), AuthService::get_key_pair()?);

        let res = auth_service.validate_jwt(String::from("testing"), Some(Duration::from_secs(10)));

        assert!(res.is_err());

        Ok(())
    }
}
