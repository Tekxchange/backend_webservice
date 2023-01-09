use super::user::MinUserReturnDto;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductDetails {
    pub description: String,
    pub title: String,
    pub price: Decimal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductReturn {
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub created_by: MinUserReturnDto,
}
