

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player{
    pub id : uuid::Uuid,
    pub name : String
}

impl Player{

    pub fn new(username : String) -> Player{
        let id = uuid::Uuid::new_v4();
        let player_id = id.to_string().to_uppercase();
        let len = player_id.len();
        let id_chars = &player_id[len-4..];

        let name = username + "#" + id_chars;
        Player{
            id,
            name,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Game{
    pub id : uuid::Uuid,
    pub name : String,
    pub map : String,
    pub mode : String,
    pub max_players : u8,
    // TODO: Add chat
}

impl Game{
    
    pub fn new(name : String, map : String, mode : String, max_players : u8) -> Game{
        Game{
            id : uuid::Uuid::new_v4(),
            name,
            map,
            mode,
            max_players
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum PlayerType{
    Host,
    Player(bool)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerGame{
    pub player_id : uuid::Uuid,
    pub game_id : uuid::Uuid,
    pub player_type : PlayerType,
}

impl PlayerGame{

    pub fn new_player(player_id : uuid::Uuid, game_id : uuid::Uuid) -> PlayerGame
    {
        PlayerGame{
            player_id,
            game_id,
            player_type : PlayerType::Player(false),
        }
    }

    pub fn new_host(player_id : uuid::Uuid, game_id : uuid::Uuid) -> PlayerGame
    {
        PlayerGame{
            player_id,
            game_id,
            player_type : PlayerType::Host,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameSem{
    pub game_id : uuid::Uuid,
    pub sem : std::sync::Arc<std::sync::Condvar>,
    pub mutex : std::sync::Arc<std::sync::Mutex<i32>>,
}

impl GameSem{

    pub fn new(game_id : uuid::Uuid) -> GameSem{
        GameSem{
            game_id,
            sem : std::sync::Arc::new(std::sync::Condvar::new()),
            mutex : std::sync::Arc::new(std::sync::Mutex::new(1)),
        }
    }
}

