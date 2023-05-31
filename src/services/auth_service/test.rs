use mockall::automock;

use crate::{db::test::establish_connection, redis::{__mock_MockRedisConnection, MockRedisConnection}};

use super::*;

type E = Result<(), Box<dyn std::error::Error>>;

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

#[tokio::test]
async fn generate_refresh_token() -> E {
    let db = establish_connection().await?;
    let key = AuthService::get_key_pair()?;
    let r = MockRedisConnection::default();

    Ok(())
}
