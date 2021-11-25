
use std::{collections::HashMap, sync::{Arc, Mutex}};
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub struct Table<T>
{
    map : Arc<Mutex<HashMap<uuid::Uuid, T>>>,
}

impl<T: Clone> Table<T>{

    pub fn new() -> Self{
        Table{
            map : Arc::new(Mutex::new(HashMap::<uuid::Uuid, T>::new())),
        }
    }

    pub fn get(&self, id : &uuid::Uuid) -> Option<T>{
        let map = self.lock();
        let val = map.get(id);
        if let Some(val) = val{
            return Some(val.clone())
        }
    
        None
    }

    pub fn insert(&self, id : uuid::Uuid, entry : T) 
    {
        self.lock().insert(id, entry);
    }

    pub fn remove(&self, id : &uuid::Uuid)
    {
        self.lock().remove(id);
    }

    pub fn get_all(&self) -> Vec<T>{
        let copy = self.lock().clone();
        copy.into_values().collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<HashMap<uuid::Uuid, T>>{
        self.map.lock().expect("Error on locking")
    }
}

pub type PlayerTable = Table<Player>;

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

pub type GameTable = Table<Game>;

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

pub type PlayerGameTable = Table<PlayerGame>;