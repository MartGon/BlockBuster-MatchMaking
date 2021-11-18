
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

    pub fn get(self, id : &uuid::Uuid) -> Option<T>{
        let map = &*self.lock();
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
}

impl Player{

    pub fn new(name : String) -> Player{
        Player{
            name
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