

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player{
    pub name : String,
    pub ready : bool
}

impl Player{

    pub fn new(name : String) -> Player{
        Player{
            name,
            ready : false
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Game{
    pub id : uuid::Uuid,
    pub name : String,
}

impl Game{
    
    pub fn new(name : String) -> Game{
        Game{
            id : uuid::Uuid::new_v4(),
            name
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlayerGame{
    pub player_id : uuid::Uuid,
    pub game_id : uuid::Uuid,
    pub ready : bool
}

impl PlayerGame{

    pub fn new(player_id : uuid::Uuid, game_id : uuid::Uuid) -> PlayerGame
    {
        PlayerGame{
            player_id,
            game_id,
            ready : false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameSem{
    pub game_id : uuid::Uuid,
    pub sem : std::sync::Arc<std::sync::Condvar>,
}

impl GameSem{

    pub fn new(game_id : uuid::Uuid) -> GameSem{
        GameSem{
            game_id,
            sem : std::sync::Arc::new(std::sync::Condvar::new())
        }
    }
}

