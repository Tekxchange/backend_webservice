use crate::models::user::UserRegister;
use crate::{create_rocket_instance, rocket};
use crate::{db::test::establish_connection, services::UserService};
use rocket::{
    http::{ContentType, Status},
    local::asynchronous::Client,
};
use serde_json::json;

#[tokio::test]
async fn test_username_exists_true() -> anyhow::Result<()> {
    let memory_db = establish_connection().await?;

    let user_service = UserService::new(memory_db.clone());

    user_service
        .create_user(
            crate::models::user::UserRegister {
                username: String::from("testUser"),
                email: String::from("test@email.com"),
                password: String::from("testPassword"),
            },
            true,
        )
        .await?;

    let rocket = create_rocket_instance(Some(memory_db)).await?;
    let client = Client::tracked(rocket).await?;

    let req = client
        .post(uri!("/api/users/username_exists"))
        .body(serde_json::json!({"username": "testUser"}).to_string())
        .header(ContentType::JSON);

    let res = req.dispatch().await;

    assert_eq!(Status::Ok, res.status());

    let res_body = res.into_string().await;

    assert_eq!(serde_json::json!(true).to_string(), res_body.unwrap());

    Ok(())
}

#[tokio::test]
async fn test_username_exists_false() -> anyhow::Result<()> {
    let rocket = create_rocket_instance(None).await?;
    let client = Client::tracked(rocket).await?;

    let mut req = client
        .post(uri!("/api/users/username_exists"))
        .body(serde_json::json!({"username": "testUser"}).to_string());

    req.add_header(ContentType::JSON);

    let res = req.dispatch().await;

    assert_eq!(Status::Ok, res.status());

    let res_body = res.into_string().await;

    assert_eq!(serde_json::json!(false).to_string(), res_body.unwrap());

    Ok(())
}

#[tokio::test]
async fn test_email_exists_true() -> anyhow::Result<()> {
    let db = establish_connection().await?;
    let user_service = UserService::new(db.clone());

    user_service
        .create_user(
            UserRegister {
                email: String::from("test@email.com"),
                password: String::from("somePassword"),
                username: String::from("testUser"),
            },
            true,
        )
        .await?;

    let rocket = create_rocket_instance(Some(db)).await?;
    let client = Client::tracked(rocket).await?;

    let request = client
        .post("/api/users/email_exists")
        .body(json!({"email": "test@email.com"}).to_string())
        .header(ContentType::JSON);

    let res = request.dispatch().await;

    assert_eq!(Status::Ok, res.status());
    assert_eq!(json!(true).to_string(), res.into_string().await.unwrap());

    Ok(())
}

#[tokio::test]
async fn test_email_exists_false() -> anyhow::Result<()> {
    let rocket = create_rocket_instance(None).await?;
    let client = Client::tracked(rocket).await?;

    let request = client
        .post("/api/users/email_exists")
        .body(json!({"email": "test@email.com"}).to_string())
        .header(ContentType::JSON);

    let res = request.dispatch().await;

    assert_eq!(Status::Ok, res.status());
    assert_eq!(json!(false).to_string(), res.into_string().await.unwrap());

    Ok(())
}
