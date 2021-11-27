
use super::entity;

use std::{collections::HashMap, sync::{Arc, Mutex}};

pub type GameTable = Table<entity::Game>;
pub type PlayerTable = Table<entity::Player>;
pub type PlayerGameTable = Table<entity::PlayerGame>;
pub type GameSemTable = Table<entity::GameSem>;

#[derive(Clone)]
pub struct DB{
    pub player_table : PlayerTable,
    pub game_table : GameTable,
    pub player_game_table : PlayerGameTable,
    pub game_sem_table :  GameSemTable,
}

impl DB{
    pub fn new() -> Self{
        DB{
            player_table : PlayerTable::new(),
            game_table : GameTable::new(),
            player_game_table : PlayerGameTable::new(),
            game_sem_table : GameSemTable::new(),
        }
    }
}

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

    pub fn remove(&self, id : &uuid::Uuid) -> Option<T>
    {
        self.lock().remove(id)
    }

    pub fn get_all(&self) -> Vec<T>{
        let copy = self.lock().clone();
        copy.into_values().collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<HashMap<uuid::Uuid, T>>{
        self.map.lock().expect("Error on locking")
    }
}