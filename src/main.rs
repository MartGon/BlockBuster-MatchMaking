use warp::{Filter};

mod matchmaking;

use matchmaking::endpoints;

#[tokio::main]
async fn main() {
    
    let player_table = matchmaking::entity::PlayerTable::new();
    let game_table = matchmaking::entity::GameTable::new();
    let player_game_table = matchmaking::entity::PlayerGameTable::new();

    let name = "Sample".to_string();
    let sample_game = matchmaking::entity::Game::new(name);
    game_table.insert(sample_game.id.clone(), sample_game);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    
    let routes = endpoints::filters::get_routes(player_table, game_table, player_game_table).or(hello);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
