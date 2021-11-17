use std::{collections::HashMap, sync::{Arc, Mutex}};
use warp::{Filter};

mod entity;
mod payload;

type Players = HashMap<uuid::Uuid, entity::Player>;

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

async fn login(player : payload::request::Login, player_table : PlayerTable) -> Result<impl warp::Reply, warp::Rejection>{
    
    let player_uuid : uuid::Uuid = uuid::Uuid::new_v4();
    let player_id = player_uuid.to_string().to_uppercase();
    let len = player_id.len();
    let id_chars = &player_id[len-4..];

    let username = player.username + "#" + id_chars;
    let player = entity::Player::new(username.clone());
    let mut guard = player_table.players.lock().expect("Error while locking player_table");
    guard.insert(player_uuid, player);

    let response = payload::response::Login{username};
    Ok(warp::reply::json(&response))
}

fn json_body() -> impl Filter<Extract = (payload::request::Login,), Error = warp::Rejection> + Clone {
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
