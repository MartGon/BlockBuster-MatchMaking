use warp::{Filter};

mod matchmaking;


#[tokio::main]
async fn main() {
    
    let player_table = matchmaking::entity::PlayerTable::new();
    let game_table = matchmaking::entity::GameTable::new();

    let name = "Sample".to_string();
    let sample_game = matchmaking::entity::Game::new(name);
    game_table.insert(sample_game.id.clone(), sample_game);

    let filter = warp::any().map(move || player_table.clone());
    let login = warp::post()
    .and(warp::path("login"))
    .and(warp::path::end())
    .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<matchmaking::payload::request::Login>()))
    .and(filter.clone())
    .and_then(matchmaking::endpoints::login);

    let list_filter = warp::any().map(move || game_table.clone());
    let list_games = warp::post()
    .and(warp::path("list_games"))
    .and(warp::path::end())
    .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<matchmaking::payload::request::ListGames>()))
    .and(list_filter.clone())
    .and_then(matchmaking::endpoints::list_games);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let routes = login.or(list_games).or(hello);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
