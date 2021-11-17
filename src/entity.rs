
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player{
    pub name : String,
}

impl Player{

    pub fn new(name : String) -> Player{
        Player{
            name
        }
    }
}