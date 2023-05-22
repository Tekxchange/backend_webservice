use rocket::{catch, serde::json::Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    error: String,
}

#[catch(401)]
pub fn unauthorized() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: "Unable to authenticate. Ensure your token is refreshed or log in again.".into(),
    })
}

#[catch(500)]
pub fn internal_error() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: "An unknown server error has occurred.".into(),
    })
}

#[catch(404)]
pub fn not_found() -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: "The requested resource is not found".into(),
    })
}
