use crate::schema::user;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Queryable)]
#[diesel(table_name = user)]
pub struct User {
    #[diesel(sql_type = BigInt)]
    pub id: i64,
    #[diesel(sql_type = Timestamp)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Timestamp)]
    pub updated_at: DateTime<Utc>,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = user)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub password: String,
}
