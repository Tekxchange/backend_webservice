use crate::{
    dtos::product::FileResponder,
    models::user::AuthUser,
    services::{FileService, FileServiceError},
};
use anyhow::anyhow;
use rocket::{data::ToByteUnit, http::ContentType, Data, Route};

#[post("/upload?<product_id>", data = "<data>")]
async fn upload_file(
    user: AuthUser,
    file_service: FileService,
    data: Data<'_>,
    content_type: &ContentType,
    product_id: i64
) -> Result<(), FileServiceError> {
    let data = data.open(10.megabytes());
    let data_bytes: Vec<u8> = data
        .into_bytes()
        .await
        .map_err(|e| FileServiceError::Unknown(crate::AnyhowResponder(anyhow!(e))))?
        .into_inner();

    let extension =
        content_type
            .0
            .extension()
            .ok_or(FileServiceError::Unknown(crate::AnyhowResponder(anyhow!(
                "Unknown incoming file extension"
            ))))?;

    file_service
        .create_file_data(user, &data_bytes, extension.as_str(), product_id)
        .await?;

    Ok(())
}

#[get("/get_file?<id>")]
async fn get_file(id: i64, file_service: FileService) -> Result<FileResponder, FileServiceError> {
    let file = file_service.get_file_data(id).await?;
    Ok(file)
}

pub fn routes() -> Vec<Route> {
    routes![upload_file, get_file]
}
