


pub mod handlers
{
    use crate::matchmaking::payload;
    use crate::matchmaking::entity;

    // /login
    pub async fn login(player : payload::request::Login, player_table : entity::PlayerTable) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let player_uuid : uuid::Uuid = uuid::Uuid::new_v4();
        let player_id = player_uuid.to_string().to_uppercase();
        let len = player_id.len();
        let id_chars = &player_id[len-4..];

        let username = player.username + "#" + id_chars;
        let player = entity::Player::new(username.clone());
        player_table.insert(player_uuid, player);

        let response = payload::response::Login{username};
        Ok(warp::reply::json(&response))
    }

    pub async fn list_games(_game_filter : serde_json::Value, game_table : entity::GameTable) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let games = game_table.get_all();

        let response = payload::response::ListGames{games};
        Ok(warp::reply::json(&response))
    }

    pub async fn create_game(create_game_req : payload::request::CreateGame, game_table : entity::GameTable)
        -> Result<impl warp::Reply, warp::Rejection>
    {
        let game = entity::Game::new(create_game_req.name);
        game_table.insert(game.id.clone(), game);

        Ok(warp::reply::with_status("", warp::http::StatusCode::OK))
    }
}

pub mod filters
{
    use warp::Filter;
    use super::handlers;
    use crate::matchmaking::payload::request;
    use crate::matchmaking::entity::*;

    pub fn get_routes(player_table : Table::<Player>, game_table : Table::<Game>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        login(player_table.clone())
        .or(list_games(game_table.clone()))
        .or(create_game(game_table.clone()))
    }

    pub fn login(player_table : Table::<Player>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || player_table.clone());

        warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<request::Login>()))
        .and(filter.clone())
        .and_then(handlers::login)
    }

    pub fn list_games(game_table : Table::<Game>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || game_table.clone());
        warp::post()
        .and(warp::path("list_games"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<serde_json::Value>()))
        .and(filter.clone())
        .and_then(handlers::list_games)
    }

    pub fn create_game(game_table : Table::<Game>) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || game_table.clone());
         warp::post()
        .and(warp::path("create_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::CreateGame>())
        .and(filter.clone())
        .and_then(handlers::create_game)
    }
}