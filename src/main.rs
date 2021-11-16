use std::{collections::HashMap, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};
use warp::Filter;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Player{
    name : String,
}

struct ID
{

}

type Players = HashMap<String, Player>;

#[derive(Clone)]
struct PlayerTable
{
    pub players : Arc<Mutex<Players>>,
}

impl PlayerTable {
    fn new() -> Self{
        PlayerTable{
            players: Arc::new(Mutex::new(Players::new())),
        }
    }
}

async fn login(player : Player, player_table : PlayerTable) -> Result<impl warp::Reply, warp::Rejection>{
    let mut guard = player_table.players.lock().expect("Error while locking player_table");
    guard.insert(player.name.clone(), player);

    let msg = format!("Login was succesful. {} players are logged", guard.len());
    Ok(warp::reply::with_status(msg, warp::http::StatusCode::OK))
}

fn json_body() -> impl Filter<Extract = (Player,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    
    let player_table = PlayerTable::new();
    let filter = warp::any().map(move || player_table.clone());

    let login = warp::post()
    .and(warp::path("login"))
    .and(warp::path::end())
    .and(json_body())
    .and(filter.clone())
    .and_then(login);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let routes = login.or(hello);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
