

pub mod handlers
{
    use std::convert::Infallible;

    use crate::matchmaking::payload;
    use crate::matchmaking::entity;
    use crate::matchmaking::database;

    use warp::reply;
    use warp::http::StatusCode;

    // /login
    pub async fn login(player : payload::request::Login, db : database::DB) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let player_uuid : uuid::Uuid = uuid::Uuid::new_v4();
        let player_id = player_uuid.to_string().to_uppercase();
        let len = player_id.len();
        let id_chars = &player_id[len-4..];

        let username = player.username + "#" + id_chars;
        let player = entity::Player::new(username.clone());
        db.player_table.insert(player_uuid, player);

        let response = payload::response::Login{id : player_uuid, username};
        Ok(warp::reply::json(&response))
    }

    pub async fn list_games(_game_filter : serde_json::Value, db : database::DB) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let games = db.game_table.get_all();

        let response = payload::response::ListGames{games};
        Ok(warp::reply::json(&response))
    }

    pub async fn join_game(join_game_req : payload::request::JoinGame, db : database::DB)
        -> Result<impl warp::Reply, Infallible>
    {
        // Find game
        if let Some(game) = db.game_table.get(&join_game_req.game_id)
        {
            // TODO: Check if the game is full

            
            // Find player
            if let Some(_player) = db.player_table.get(&join_game_req.player_id)
            {
                // Insert new entry
                let player_game = entity::PlayerGame::new(join_game_req.player_id, join_game_req.game_id);
                db.player_game_table.insert(join_game_req.player_id, player_game);

                // Send response
                let response = get_game_details(db, game);
                return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
            }

            let err = format!("Could not find player with id {}", join_game_req.player_id.to_string());
            return Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
        }

        let err = format!("Could not find game with id {}", join_game_req.game_id.to_string());
        Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    pub async fn create_game(create_game_req : payload::request::CreateGame, db : database::DB)
        -> Result<impl warp::Reply, warp::Rejection>
    {
        let game = entity::Game::new(create_game_req.name);
        db.game_table.insert(game.id.clone(), game.clone());

        let game_sem = entity::GameSem::new(game.id);
        db.game_sem_table.insert(game.id, game_sem);

        Ok(warp::reply::with_status("", warp::http::StatusCode::OK))
    }

    pub async fn leave_game(leave_game_req : payload::request::LeaveGame, db : database::DB)
        -> Result<impl warp::Reply, Infallible>
    {
        let player_id = leave_game_req.player_id;
        
        if let Some(entry)  = db.player_game_table.remove(&player_id)
        {
            if let Some(game_sem) = db.game_sem_table.get(&entry.game_id)
            {
                game_sem.sem.notify_all();
            }
        }

        Ok(reply::with_status(reply::json(&"".to_string()), StatusCode::OK))
    }

    pub async fn toggle_ready(toggle_ready_req : payload::request::ToggleReady, db : database::DB)
        -> Result<impl warp::Reply, Infallible>
    {
        let player_id = toggle_ready_req.player_id;
        if let Some(mut player) = db.player_table.get(&player_id)
        {
            if let Some(_game_id)  = db.player_game_table.get(&player_id)
            {
                player.ready = !player.ready;
                let response = serde_json::json!({"ready" : player.ready});
                db.player_table.insert(player_id, player);
                
                return Ok(reply::with_status(reply::json(&response), StatusCode::OK));
            }
            
            let err = format!("Player was not in a game {}", player_id.to_string());
            return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
        }

        let err = format!("Could not find player with id {}", player_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    pub async fn update_game(update_game_req : payload::request::UpdateGame, db : database::DB) -> Result<impl warp::Reply, Infallible>
    {
        let game_id = update_game_req.game_id;

        if let Some(game) = db.game_table.get(&game_id)
        {
            let game_sem = db.game_sem_table.get(&game_id).expect("Inconsistency: Game had no lock");
            let dummy_mutex = std::sync::Mutex::new(1);
            let _game_sem = game_sem.sem.wait(dummy_mutex.lock().unwrap()).unwrap();

            let response = get_game_details(db, game);
            return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
        }

        let err = format!("Could not find game with id {}", game_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    fn get_game_details(db : database::DB, game : entity::Game) -> payload::response::GameDetails
    {
        // Get in game players
        let mut game_players = Vec::<entity::Player>::new();
        for entry in db.player_game_table.get_all().into_iter()
        {
            let game_id = entry.game_id;
            
            if game_id == game.id
            {
                let player = db.player_table.get(&entry.player_id).expect("Could not find player in playertable");
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
    use crate::matchmaking::database;

    pub fn get_routes(db : database::DB) 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        login(db.clone())
        .or(list_games(db.clone()))
        .or(create_game(db.clone()))
        .or(join_game(db.clone()))
        .or(leave_game(db.clone()))
        .or(toggle_ready(db.clone()))
        .or(update_game(db.clone()))
    }

    pub fn login(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());

        warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<request::Login>()))
        .and(filter.clone())
        .and_then(handlers::login)
    }

    pub fn list_games(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        warp::post()
        .and(warp::path("list_games"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<serde_json::Value>()))
        .and(filter.clone())
        .and_then(handlers::list_games)
    }

    pub fn join_game(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());

        warp::post()
        .and(warp::path("join_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16).and(warp::body::json::<request::JoinGame>()))
        .and(filter.clone())
        .and_then(handlers::join_game)
    }

    pub fn create_game(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());

        warp::post()
        .and(warp::path("create_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::CreateGame>())
        .and(filter.clone())
        .and_then(handlers::create_game)
    }

    pub fn leave_game(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        
        warp::post()
        .and(warp::path("leave_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::LeaveGame>())
        .and(filter.clone())
        .and_then(handlers::leave_game)
    }

    pub fn toggle_ready(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        
        warp::post()
        .and(warp::path("toggle_ready"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::ToggleReady>())
        .and(filter.clone())
        .and_then(handlers::toggle_ready)
    }

    pub fn update_game(db : database::DB)
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());

        warp::post()
        .and(warp::path("update_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::UpdateGame>())
        .and(filter.clone())
        .and_then(handlers::update_game)
    }
}