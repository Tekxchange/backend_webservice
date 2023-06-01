pub use super::UserService;
use crate::{db::test::establish_connection, models::user::UserRegister};
use sea_orm::prelude::*;

type E = Result<(), Box<dyn std::error::Error>>;

mod create_user {
    use super::*;

    fn create_test_user(name: &str, pass: &str) -> UserRegister {
        UserRegister {
            username: name.to_owned(),
            email: format!("{name}@example.com"),
            password: pass.to_owned(),
        }
    }

    #[tokio::test]
    async fn successful_creation() -> E {
        let db = establish_connection().await?;
        let user_service = UserService::new(db.clone());
        let test_user = create_test_user("test", "password");
        let created = user_service.create_user(test_user.clone(), false).await?;

        let found = entity::user::Entity::find_by_id(created).one(&db).await?;

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.email, test_user.email);
        assert_eq!(found.username, test_user.username);
        assert_ne!(found.password, test_user.password);

        Ok(())
    }

    #[tokio::test]
    async fn unsuccessful_bad_username() -> E {
        let db = establish_connection().await?;
        let user = create_test_user("twatwaffle", "testpass");
        let user_service = UserService::new(db);

        let result = user_service.create_user(user, false).await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn unsuccessful_existing_username() -> E {
        let db = establish_connection().await?;
        let user_service = UserService::new(db);
        let test_user = create_test_user("testName", "testPass");
        assert!(user_service
            .create_user(test_user.clone(), false)
            .await
            .is_ok());
        let second = user_service.create_user(test_user, false).await;
        assert!(second.is_err());
        Ok(())
    }
}

mod get_by_email {

    use super::*;

    #[tokio::test]
    async fn successfully_gets_by_email_ignoring_case() -> E {
        let db = establish_connection().await?;
        let user_service = UserService::new(db);
        user_service
            .create_user(
                UserRegister {
                    username: String::from("test"),
                    email: String::from("test@test.com"),
                    password: String::from("testpass"),
                },
                true,
            )
            .await?;

        let res = user_service.get_by_email("test@test.com").await?;
        assert!(res.is_some());
        let res = user_service.get_by_email("TeSt@TeSt.CoM").await?;
        assert!(res.is_some());
        let res = user_service.get_by_email("test").await?;
        assert!(res.is_none());

        Ok(())
    }
}

mod get_by_username {
    use super::*;

    #[tokio::test]
    async fn successfully_gets_by_username_ignoring_case() -> E {
        let db = establish_connection().await?;
        let user_service = UserService::new(db);
        user_service
            .create_user(
                UserRegister {
                    username: "testing123".into(),
                    email: "test@test.com".into(),
                    password: "testPass".into(),
                },
                false,
            )
            .await?;

        let res = user_service.get_by_username("testing123").await?;
        assert!(res.is_some());
        let res = user_service.get_by_username("TeStInG123").await?;
        assert!(res.is_some());
        let res = user_service.get_by_username("testing").await?;
        assert!(res.is_none());

        Ok(())
    }
}

mod get_by_id {
    use super::*;

    #[tokio::test]
    async fn successfully_gets_user_by_id() -> E {
        let db = establish_connection().await?;
        let user_service = UserService::new(db);
        let user_id = user_service
            .create_user(
                UserRegister {
                    username: "test".into(),
                    email: "test@test.com".into(),
                    password: "testing".into(),
                },
                false,
            )
            .await?;

        let found = user_service.get_user_by_id(&user_id).await?;
        assert!(found.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn returns_none_for_not_found_id() -> E {
        let db = establish_connection().await?;
        let user_service = UserService::new(db);

        let found = user_service.get_user_by_id(&1).await?;
        assert!(found.is_none());

        Ok(())
    }
}
