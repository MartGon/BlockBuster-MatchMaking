


pub mod handlers
{
    use std::convert::Infallible;

    use crate::matchmaking::entity::GameSemTable;
    use crate::matchmaking::payload;
    use crate::matchmaking::entity;

    use warp::reply;
    use warp::http::StatusCode;

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

        let response = payload::response::Login{id : player_uuid, username};
        Ok(warp::reply::json(&response))
    }

    pub async fn list_games(_game_filter : serde_json::Value, game_table : entity::GameTable) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let games = game_table.get_all();

        let response = payload::response::ListGames{games};
        Ok(warp::reply::json(&response))
    }

    pub async fn join_game(join_game_req : payload::request::JoinGame, 
        game_table : entity::GameTable, 
        player_table : entity::PlayerTable, 
        player_game_table : entity::PlayerGameTable)
        -> Result<impl warp::Reply, Infallible>
    {
        // Find game
        if let Some(game) = game_table.get(&join_game_req.game_id)
        {
            // TODO: Check if the game is full

            
            // Find player
            if let Some(_player) = player_table.get(&join_game_req.player_id)
            {
                // Insert new entry
                let player_game = entity::PlayerGame::new(join_game_req.player_id, join_game_req.game_id);
                player_game_table.insert(join_game_req.player_id, player_game);

                // Send response
                let response = get_game_details(game, player_table, player_game_table);
                return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
            }

            let err = format!("Could not find player with id {}", join_game_req.player_id.to_string());
            return Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
        }

        let err = format!("Could not find game with id {}", join_game_req.game_id.to_string());
        Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    pub async fn create_game(create_game_req : payload::request::CreateGame, 
        game_table : entity::GameTable,
        game_sem_table : GameSemTable)
        -> Result<impl warp::Reply, warp::Rejection>
    {
        let game = entity::Game::new(create_game_req.name);
        game_table.insert(game.id.clone(), game.clone());

        let game_sem = entity::GameSem::new(game.id);
        game_sem_table.insert(game.id, game_sem);

        Ok(warp::reply::with_status("", warp::http::StatusCode::OK))
    }

    pub async fn leave_game(leave_game_req : payload::request::LeaveGame, 
        player_game_table : entity::PlayerGameTable,
        game_sem_table : GameSemTable)
        -> Result<impl warp::Reply, Infallible>
    {
        let player_id = leave_game_req.player_id;
        
        if let Some(entry)  = player_game_table.remove(&player_id)
        {
            if let Some(game_sem) = game_sem_table.get(&entry.game_id)
            {
                game_sem.sem.notify_all();
            }
        }

        Ok(reply::with_status(reply::json(&"".to_string()), StatusCode::OK))
    }

    pub async fn toggle_ready(toggle_ready_req : payload::request::ToggleReady, 
        player_table : entity::PlayerTable,
        player_game_table : entity::PlayerGameTable)
        -> Result<impl warp::Reply, Infallible>
    {
        let player_id = toggle_ready_req.player_id;
        if let Some(mut player) = player_table.get(&player_id)
        {
            if let Some(_game_id)  = player_game_table.get(&player_id)
            {
                player.ready = !player.ready;
                let response = serde_json::json!({"ready" : player.ready});
                player_table.insert(player_id, player);
                
                return Ok(reply::with_status(reply::json(&response), StatusCode::OK));
            }
            
            let err = format!("Player was not in a game {}", player_id.to_string());
            return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
        }

        let err = format!("Could not find player with id {}", player_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    pub async fn update_game(update_game_req : payload::request::UpdateGame,
        game_table : entity::GameTable,
        game_sem_table : entity::GameSemTable,
        player_table : entity::PlayerTable,
        player_game_table : entity::PlayerGameTable) -> Result<impl warp::Reply, Infallible>
    {
        let game_id = update_game_req.game_id;

        if let Some(game) = game_table.get(&game_id)
        {
            let game_sem = game_sem_table.get(&game_id).expect("Inconsistency: Game had no lock");
            let dummy_mutex = std::sync::Mutex::new(1);
            let _game_sem = game_sem.sem.wait(dummy_mutex.lock().unwrap()).unwrap();

            let response = get_game_details(game, player_table, player_game_table);
            return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
        }

        let err = format!("Could not find game with id {}", game_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    fn get_game_details(game : entity::Game, 
        player_table : entity::PlayerTable,
        player_game_table : entity::PlayerGameTable) -> payload::response::GameDetails
    {
        // Get in game players
        let mut game_players = Vec::<entity::Player>::new();
        for entry in player_game_table.get_all().into_iter()
        {
            let game_id = entry.game_id;
            
            if game_id == game.id
            {
                let player = player_table.get(&entry.player_id).expect("Could not find player in playertable");
                game_players.push(player);
            }
        }

        payload::response::GameDetails{id : game.id, name : game.name, players :  game_players}
    }
}

pub mod filters
{
    use warp::Filter;
    use super::handlers;
    use crate::matchmaking::payload::request;
    use crate::matchmaking::entity::*;

    pub fn get_routes(player_table : PlayerTable, game_table : GameTable, player_game_table : PlayerGameTable, game_sem_table : Table<GameSem>) 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        login(player_table.clone())
        .or(list_games(game_table.clone()))
        .or(create_game(game_table.clone(), game_sem_table.clone()))
        .or(join_game(game_table.clone(), player_table.clone(), player_game_table.clone()))
        .or(leave_game(player_game_table.clone(), game_sem_table.clone()))
        .or(toggle_ready(player_table.clone(), player_game_table.clone()))
        .or(update_game(player_table, game_table, player_game_table, game_sem_table))
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

    pub fn join_game(game_table : Table::<Game>, player_table : Table::<Player>, player_game_table : PlayerGameTable) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any()
        .map(move || game_table.clone())
        .and(warp::any().map(move || player_table.clone())
        .and(warp::any().map(move || player_game_table.clone())));

        warp::post()
        .and(warp::path("join_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<request::JoinGame>()))
        .and(filter.clone())
        .and_then(handlers::join_game)
    }

    pub fn create_game(game_table : Table::<Game>, game_sem_table : GameSemTable) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || game_table.clone())
        .and(warp::any().map(move || game_sem_table.clone()));

        warp::post()
        .and(warp::path("create_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::CreateGame>())
        .and(filter.clone())
        .and_then(handlers::create_game)
    }

    pub fn leave_game(player_game_table : PlayerGameTable, game_sem_table : GameSemTable) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any()
        .map(move || player_game_table.clone())
        .and(warp::any().map(move || game_sem_table.clone()));
        
        warp::post()
        .and(warp::path("leave_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::LeaveGame>())
        .and(filter.clone())
        .and_then(handlers::leave_game)
    }

    pub fn toggle_ready(player_table : Table::<Player>, player_game_table : PlayerGameTable) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any()
        .map(move || player_table.clone())
        .and(warp::any().map(move || player_game_table.clone()));
        
        warp::post()
        .and(warp::path("toggle_ready"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::ToggleReady>())
        .and(filter.clone())
        .and_then(handlers::toggle_ready)
    }

    pub fn update_game(player_table : PlayerTable, game_table : GameTable, player_game_table : PlayerGameTable, game_sem_table : GameSemTable)
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any()
        .map(move || game_table.clone())
        .and(warp::any().map(move || game_sem_table.clone()))
        .and(warp::any().map(move || player_table.clone())
        .and(warp::any().map(move || player_game_table.clone())));

        warp::post()
        .and(warp::path("update_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::UpdateGame>())
        .and(filter.clone())
        .and_then(handlers::update_game)
    }
}