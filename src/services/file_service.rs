use crate::{dtos::product::FileResponder, models::user::AuthUser, AnyhowResponder};
use anyhow::anyhow;
use rocket::{
    http::ContentType,
    outcome::{try_outcome, IntoOutcome},
    request::FromRequest,
    Request,
};
use sea_orm::{prelude::*, ActiveValue, DatabaseConnection};
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

const MAX_PICTURES_NON_PREMIUM: u16 = 5;
const _MAX_PICTURES_PREMIUM: u16 = 20;

#[derive(Responder, Error, Debug)]
pub enum FileServiceError {
    #[response(status = 500)]
    #[error("Unable to create file data")]
    FileCreationError(AnyhowResponder),
    #[response(status = 404)]
    #[error("The requested file is not found")]
    FileNotFound(AnyhowResponder),
    #[response(status = 500)]
    #[error("An unknown error has occurred")]
    Unknown(AnyhowResponder),
    #[error("You do not have permissions to access this file")]
    #[response(status = 403)]
    NotAllowed(AnyhowResponder),
    #[error("Unable to add new pictures for product as you have reached the max already")]
    #[response(status = 400)]
    TooManyPictures(AnyhowResponder),
}

#[async_trait]
impl<'r> FromRequest<'r> for FileService {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let path: String = try_outcome!(std::env::var("SAVE_PATH").map_err(|_| ()).or_forward(()));

        req.rocket()
            .state::<DatabaseConnection>()
            .map(move |db| Self::new(db.clone(), path.into()))
            .or_forward(())
    }
}

pub struct FileService {
    db: DatabaseConnection,
    base_file_path: PathBuf,
}

impl FileService {
    pub fn new(db: DatabaseConnection, location: PathBuf) -> Self {
        Self {
            db,
            base_file_path: location,
        }
    }

    pub async fn create_file_data(
        &self,
        user: AuthUser,
        data: &[u8],
        extension: &str,
        for_product: i64,
    ) -> Result<i64, FileServiceError> {
        let file_location: PathBuf;
        loop {
            let new_key = uuid::Uuid::new_v4().to_string();
            let loc = self.base_file_path.join(&format!("{new_key}.{extension}"));
            if !loc.exists() {
                file_location = loc;
                break;
            }
        }

        let entity::product::Model {
            created_by,
            id: product_id,
            ..
        } = entity::product::Entity::find_by_id(for_product)
            .one(&self.db)
            .await
            .map_err(|e| FileServiceError::Unknown(AnyhowResponder(anyhow!(e))))?
            .ok_or(FileServiceError::Unknown(AnyhowResponder(anyhow!(
                "Product with id {for_product} not found"
            ))))?;

        if created_by != user.user.id {
            return Err(FileServiceError::NotAllowed(AnyhowResponder(anyhow!(
                "User with id {created_by} is not the owner of product {for_product}"
            ))));
        }

        let current_pic_count = entity::product_picture::Entity::find()
            .filter(entity::product_picture::Column::ProductId.eq(product_id))
            .count(&self.db)
            .await
            .map_err(|e| FileServiceError::Unknown(AnyhowResponder(anyhow!(e))))?;

        if current_pic_count >= MAX_PICTURES_NON_PREMIUM as u64 {
            return Err(FileServiceError::TooManyPictures(AnyhowResponder(anyhow!(
                "User already has {current_pic_count} pictures for product. Unable to add any new pictures."
            ))));
        }

        let path_str = file_location
            .to_str()
            .ok_or(FileServiceError::Unknown(AnyhowResponder(anyhow!(
                "Unable to convert Path to String"
            ))))?;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&file_location)
            .map_err(|e| FileServiceError::FileCreationError(AnyhowResponder(anyhow!(e))))?;

        let mut writer = BufWriter::new(file);
        writer
            .write_all(data)
            .map_err(|e| FileServiceError::FileCreationError(AnyhowResponder(anyhow!(e))))?;

        let entity::file::Model { id: file_id, .. } = entity::file::ActiveModel {
            created_by: ActiveValue::Set(user.user.id),
            file_location: ActiveValue::Set(path_str.to_owned()),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| FileServiceError::FileCreationError(AnyhowResponder(anyhow!(e))))?;

        entity::product_picture::ActiveModel {
            product_id: ActiveValue::Set(product_id),
            file_id: ActiveValue::Set(file_id),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| FileServiceError::FileCreationError(AnyhowResponder(anyhow!(e))))?;

        Ok(file_id)
    }

    pub async fn get_file_data(&self, id: i64) -> Result<FileResponder, FileServiceError> {
        let found_file = entity::file::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| FileServiceError::Unknown(AnyhowResponder(anyhow!(e))))?
            .ok_or(FileServiceError::FileNotFound(AnyhowResponder(anyhow!(
                "File with id {id} not found"
            ))))?;

        let location = Path::new(&found_file.file_location);
        if !location.exists() {
            return Err(FileServiceError::Unknown(AnyhowResponder(anyhow!(
                "File with id {id} exists in db, but not on the file system"
            ))));
        }

        let file = OpenOptions::new()
            .read(true)
            .open(location)
            .map_err(|e| FileServiceError::Unknown(AnyhowResponder(anyhow!(e))))?;
        let content_type = location
            .extension()
            .map(|ext| ext.to_str().map(|s| ContentType::from_extension(s)))
            .flatten()
            .flatten()
            .ok_or(FileServiceError::Unknown(AnyhowResponder(anyhow!(
                "Unable to get file extension from path"
            ))))?;

        Ok(FileResponder { file, content_type })
    }
}
