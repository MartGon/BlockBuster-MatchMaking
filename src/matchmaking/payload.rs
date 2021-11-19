
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
    pub struct CreateGame
    {
        pub name : String,
    }
}

pub mod response
{
    use serde::{Deserialize, Serialize};

    use crate::matchmaking::entity;
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Login
    {
        pub username : String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ListGames{
        pub games : Vec<entity::Game>,
    }
}