use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    Admin,
    Moderator,
    User,
}
