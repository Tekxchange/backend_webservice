use crate::{
    dtos::product::FileResponder,
    models::user::AuthUser,
    services::{FileService, FileServiceError},
};
use anyhow::anyhow;
use rocket::{form::Form, fs::TempFile, Route};

#[derive(FromForm, Debug)]
struct UploadData<'a> {
    data: TempFile<'a>,
}

#[tracing::instrument(level = "trace")]
#[post("/upload?<product_id>", data = "<data>")]
async fn upload_file<'a>(
    user: AuthUser,
    file_service: FileService,
    mut data: Form<Option<UploadData<'a>>>,
    product_id: i64,
) -> Result<(), FileServiceError> {
    let data = data
        .take()
        .ok_or(FileServiceError::FileCreationError(crate::AnyhowResponder(
            anyhow!("No files found to process"),
        )))?;

    let file = data.data;

    file_service
        .create_file_data(user, file, product_id)
        .await?;

    Ok(())
}

#[tracing::instrument(level = "trace")]
#[get("/get_file?<id>")]
async fn get_file(id: i64, file_service: FileService) -> Result<FileResponder, FileServiceError> {
    let file = file_service.get_file_data(id).await?;
    Ok(file)
}

pub fn routes() -> Vec<Route> {
    routes![upload_file, get_file]
}
