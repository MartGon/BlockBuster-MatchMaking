
use serde::{Deserialize, Serialize};

pub mod request
{
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Login
    {
        pub username : String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ListGames
    {
        pub full : bool,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct JoinGame
    {
        pub player_id : uuid::Uuid,
        pub game_id : uuid::Uuid,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct CreateGame
    {
        pub name : String,
    }
}

pub mod response
{
    use serde::{Deserialize, Serialize};

    use crate::matchmaking::entity::{self, Player};
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Login
    {
        pub id : uuid::Uuid,
        pub username : String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ListGames{
        pub games : Vec<entity::Game>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct JoinGame
    {
        pub id : uuid::Uuid,
        pub name : String,
        pub players : Vec<Player>,
    }
}