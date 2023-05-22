use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginReturn {
    pub jwt: String,
    pub refresh_token: String,
}
