use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    User = 1 << 0,
    Moderator = 1 << 1,
    Admin = 1 << 2,
}

impl TryFrom<i16> for Role {
    type Error = ();

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Role::User),
            2 => Ok(Role::Moderator),
            4 => Ok(Role::Admin),
            _ => Err(()),
        }
    }
}
