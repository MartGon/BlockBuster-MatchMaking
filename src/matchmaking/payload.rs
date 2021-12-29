
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
        pub player_id : uuid::Uuid,
        pub name : String,
        pub map : String,
        pub mode : String,
        pub max_players : u8,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct LeaveGame
    {
        pub player_id : uuid::Uuid,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ToggleReady
    {
        pub player_id : uuid::Uuid,
    }
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct SendChatMsg
    {
        pub player_id : uuid::Uuid,
        pub msg : String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct UpdateGame
    {
        pub game_id : uuid::Uuid,
    }
}

pub mod response
{
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Login
    {
        pub id : uuid::Uuid,
        pub username : String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GameInfo{
        pub id : uuid::Uuid,
        pub name : String,
        pub map : String,
        pub mode : String,
        pub max_players : u8,
        pub players : u8,
        pub ping : u16,
        pub chat : Vec<String>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct ListGames{
        pub games : Vec<GameInfo>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct PlayerInfo{
        pub name : String,
        pub ready : bool,
        pub host : bool
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct GameDetails
    {
        pub game_info : GameInfo,
        pub players : Vec<PlayerInfo>,
    }
}