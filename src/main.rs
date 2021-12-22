use warp::{Filter};

mod matchmaking;

use matchmaking::endpoints;
use matchmaking::entity;
use matchmaking::database;


#[tokio::main]
async fn main() {

    let db = database::DB::new();

    let name = "Sample".to_string();
    let map = "Kobra".to_string();
    let mode = "DeathMatch".to_string();
    let max_players : u8 = 16;
    let sample_game = matchmaking::entity::Game::new(name, map, mode, max_players);
    let game_sem = entity::GameSem::new(sample_game.id.clone());
    db.game_table.insert(sample_game.id.clone(), sample_game.clone());
    db.game_sem_table.insert(sample_game.id, game_sem);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    
    let routes = endpoints::filters::get_routes(db).or(hello);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
