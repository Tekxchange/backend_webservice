use crate::{db::test::establish_connection, models::user::UserRegister, services::UserService};

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

        Ok(())
    }
}
