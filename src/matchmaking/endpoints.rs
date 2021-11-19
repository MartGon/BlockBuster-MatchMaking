
use super::payload;
use super::entity;

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