use crate::{
    db::establish_connection,
    models::user::{User, UserRegister},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rocket::request::{self, FromRequest};
use rocket::Request;
use sea_orm::DatabaseConnection;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("This username already exists")]
    DuplicateUsernameError,
    #[error("This email already exists")]
    DuplicateEmailError,
    #[error(transparent)]
    DbError(crate::db::DbError),
    #[error("An unknown error occurred")]
    Unknown,
}

pub struct UserService {
    db_connection: DatabaseConnection,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserService {
    type Error = UserServiceError;

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match establish_connection().await.map_err(|e| UserServiceError::DbError(e)) {
            Ok(conn) => request::Outcome::Success(Self {
                db_connection: conn,
            }),
            Err(e) => request::Outcome::Failure((rocket::http::Status::InternalServerError, e)),
        }
    }
}

impl UserService {
    pub fn create_user(&mut self, mut register: UserRegister) -> Result<(), UserServiceError> {
        // let res = diesel::sql_query("").get_result::<User>(&mut self.db_connection);

        let salt = SaltString::generate(OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(register.password.as_bytes(), &salt)
            .unwrap()
            .to_string();

        register.password = password_hash;

        // match diesel::insert_into(schema::user::table)
        //     .values(&register)
        //     .execute(&mut self.db_connection)
        // {
        //     Err(e) => match e {
        //         _ => Err(UserServiceError::Unknown),
        //     },
        //     Ok(_) => Ok(()),
        // }
        todo!()
    }
}
