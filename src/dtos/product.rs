use geolocation_utils::{Coordinate, DistanceUnit};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductFilter {
    pub coordinate: Coordinate,
    pub radius: Decimal,
    pub units: Option<DistanceUnit>,
    pub query: Option<String>,
    pub price_low: Option<Decimal>,
    pub price_high: Option<Decimal>,
    pub city: Option<String>,
    pub zip: Option<String>,
    pub product_id_lower: Option<i64>,
}
