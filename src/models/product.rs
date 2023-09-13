use super::user::MinUserReturnDto;
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetails {
    pub description: String,
    pub title: String,
    pub price: Decimal,
    pub country: String,
    pub state: String,
    pub city: String,
    pub zip: String,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductReturn {
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub created_by: MinUserReturnDto,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub pictures: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductReturnNoUser {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub price: Decimal,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub city: String,
    pub state: String,
    pub country: String,
    pub zip: String,
    pub pictures: Vec<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProductLocationReturn {
    pub id: i64,
    pub latitude: Decimal,
    pub longitude: Decimal,
}
