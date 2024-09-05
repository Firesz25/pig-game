use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Player {
    pub name: u128,
    pub score: u8,
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialEq<u128> for Player {
    fn eq(&self, other: &u128) -> bool {
        self.name == *other
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: Ulid::new().0,
            score: 0,
        }
    }
}

impl Player {
    pub fn new(name: u128) -> Self {
        Self { name, score: 0 }
    }
}
