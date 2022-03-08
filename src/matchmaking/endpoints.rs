

pub mod handlers
{
    use std::path::Path;
    use std::convert::Infallible;
    use std::time::Duration;
    use std::process::Command;

    use crate::matchmaking::entity::GameState;
    use crate::matchmaking::entity::PlayerType;
    use crate::matchmaking::payload;
    use crate::matchmaking::entity;
    use crate::matchmaking::database;

    use rand::Rng;
    use ringbuffer::RingBufferExt;
    use ringbuffer::RingBufferWrite;
    use warp::reply;
    use warp::http::StatusCode;

    // /login
    pub async fn login(player : payload::request::Login, db : database::DB) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let player = entity::Player::new(player.username.clone());
        let player_copy = player.clone();
        db.player_table.insert(player.id, player);

        let response = payload::response::Login{id : player_copy.id, username : player_copy.name};
        Ok(warp::reply::json(&response))
    }

    pub async fn list_games(_game_filter : serde_json::Value, db : database::DB) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let games = db.game_table.get_all();

        let mut game_info_list = Vec::new();
        for game in games.into_iter()
        {
            let game_info = get_game_info(&db, &game.id).unwrap();
            game_info_list.push(game_info);
        }

        let response = payload::response::ListGames{games : game_info_list};
        Ok(warp::reply::json(&response))
    }

    pub async fn create_game(cg_req : payload::request::CreateGame, db : database::DB)
    -> Result<impl warp::Reply, warp::Rejection>
    {
        let game = entity::Game::new(cg_req.name, cg_req.map, cg_req.mode, cg_req.max_players);
        db.game_table.insert(game.id.clone(), game.clone());

        let game_sem = entity::GameSem::new(game.id);
        db.game_sem_table.insert(game.id, game_sem);

        join_game_as(&db, cg_req.player_id, game.id, true).unwrap();

        // Send response
        let response = get_game_details(&db, &game.id).unwrap();
        return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
    }

    pub async fn join_game(join_game_req : payload::request::JoinGame, db : database::DB)
        -> Result<impl warp::Reply, Infallible>
    {
        let res = join_game_as(&db, join_game_req.player_id, join_game_req.game_id, false);
        if let Ok(player_game) = res
        {
            // Notify
            notify_game_update(&db, &player_game.game_id);

            // Send response
            let response = get_game_details(&db, &join_game_req.game_id).unwrap();
            return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
        }

        let mut err = format!("Could not find game with id {}", join_game_req.game_id.to_string());
        match res {
            Err(JoinGameError::GameFull) => err = format!("Game was full"),
            Err(JoinGameError::GameNotFound) => err = format!("Could not find game with id {}", join_game_req.game_id.to_string()),
            Err(JoinGameError::PlayerNotFound) => err = format!("Could not find player with id {}", join_game_req.player_id.to_string()),
            _ => {}
        }
        
        Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    // TODO: Call this after a player is AFK for a long time.
    pub async fn leave_game(leave_game_req : payload::request::LeaveGame, db : database::DB)
        -> Result<impl warp::Reply, Infallible>
    {
        let player_id = leave_game_req.player_id;
        
        if let Some(entry)  = db.player_game_table.remove(&player_id)
        {
            let game_id = &entry.game_id;
            let game_players = get_game_players(&db, game_id);
            if game_players.is_empty()
            {
                db.game_table.remove(game_id);
            }
            else
            {
                let player = game_players.first().unwrap();
                let mut new_host = db.player_game_table.get(&player.id).unwrap();
                new_host.player_type = PlayerType::Host;
                db.player_game_table.insert(player.id, new_host);
            }

            notify_game_update(&db, game_id);
        }

        Ok(reply::with_status(reply::json(&"".to_string()), StatusCode::OK))
    }

    pub async fn toggle_ready(toggle_ready_req : payload::request::ToggleReady, db : database::DB)
        -> Result<impl warp::Reply, Infallible>
    {
        let player_id = toggle_ready_req.player_id;
        if let Some(mut player_game)  = db.player_game_table.get(&player_id)
        {
            if let entity::PlayerType::Player(ready) = player_game.player_type
            {
                player_game.player_type = entity::PlayerType::Player(!ready);
                let response = serde_json::json!({"ready" : ready});
                notify_game_update(&db, &player_game.game_id);

                db.player_game_table.insert(player_id, player_game);
                
                return Ok(reply::with_status(reply::json(&response), StatusCode::OK));
            }
            
            let err = format!("Player was host. Cannot set ready {}", player_id.to_string());
            return Ok(reply::with_status(reply::json(&err), StatusCode::BAD_REQUEST));
        }
            
        let err = format!("Player was not in a game {}", player_id.to_string());
        return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
    }

    pub async fn send_chat_msg(scm_req : payload::request::SendChatMsg, db : database::DB)
    -> Result<impl warp::Reply, Infallible>
    {
        let player_id = scm_req.player_id;
        if let Some(player_game)  = db.player_game_table.get(&player_id)
        {
            let mut game = db.game_table.get(&player_game.game_id).unwrap();
            let player = db.player_table.get(&player_game.player_id).unwrap();
            let game_id = game.id.clone();

            let chat_msg = player.name + ": " + &scm_req.msg + "\n";
            game.chat.push(chat_msg);
            db.game_table.insert(game.id, game);

            notify_game_update(&db, &game_id);
            
            return Ok(reply::with_status(reply::json(&"".to_string()), StatusCode::OK));
        }
            
        let err = format!("Player was not in a game {}", player_id.to_string());
        return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
    }

    pub async fn update_game(update_game_req : payload::request::UpdateGame, db : database::DB) -> Result<impl warp::Reply, Infallible>
    {
        let game_id = update_game_req.game_id;

        if let Some(game) = db.game_table.get(&game_id)
        {
            let game_sem = db.game_sem_table.get(&game_id).expect("Inconsistency: Game had no lock");
            let _game_sem = game_sem.sem.wait_timeout(game_sem.mutex.lock().unwrap(), Duration::from_secs(15));

            if let Ok(response) = get_game_details(&db, &game.id)
            {
                return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
            }
        }

        let err = format!("Could not find game with id {}", game_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    pub async fn start_game(start_game_req : payload::request::StartGame, db : database::DB, exec_path : String) -> Result<impl warp::Reply, Infallible>
    {
        let game_id = start_game_req.game_id;

        if let Some(mut game) = db.game_table.get(&game_id)
        {
            if let GameState::InLobby = game.state
            {
                let player_id = start_game_req.player_id;
                if let Some(player_game) = db.player_game_table.get(&player_id)
                {
                    if let PlayerType::Host = player_game.player_type 
                    {
                        if let Ok((address, port)) = launch_game(&db, game_id, exec_path)
                        {
                            game.port = Some(port);
    
                            game.address = Some(String::from(address));
                            game.state = GameState::InGame;

                            db.game_table.insert(game.id, game);
                            notify_game_update(&db, &game_id);

                            return Ok(reply::with_status(reply::json(&"".to_string()), StatusCode::OK));
                        }
                    }
                    let err = format!("Player was not host");
                    return Ok(reply::with_status(reply::json(&err), StatusCode::BAD_REQUEST));
                }
                let err = format!("Could not find player {} in game with id {}", player_id.to_string(), game_id.to_string());
                return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
            }
            let err = format!("Game was not in lobby");
            return Ok(reply::with_status(reply::json(&err), StatusCode::BAD_REQUEST));
        }
        let err = format!("Could not find game with id {}", game_id.to_string());
        return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
    }

    #[derive(Debug)]
    enum JoinGameError
    {
        GameFull,
        PlayerNotFound,
        GameNotFound,
    }

    fn join_game_as(db : &database::DB, player_id : uuid::Uuid, game_id : uuid::Uuid, host : bool) -> Result<entity::PlayerGame, JoinGameError>
    {
        if let Ok(full) = is_game_full(db, &game_id)
        {
            if !full {
                // Find player
                if let Some(_player) = db.player_table.get(&player_id)
                {
                    // Insert new entry
                    let player_game = if host {entity::PlayerGame::new_host(player_id, game_id)} else {entity::PlayerGame::new_player(player_id, game_id)};
                    db.player_game_table.insert(player_id, player_game.clone());

                    return Ok(player_game);
                }
                return Err(JoinGameError::PlayerNotFound);
            }
            return Err(JoinGameError::GameFull);
        }
        Err(JoinGameError::GameNotFound)
    }

    enum LaunchServerError
    {
        ServerCrashed,
        CouldNotlaunch,
        GameNotFound
    }

    fn launch_game(db : & database::DB, game_id : uuid::Uuid, exec_path : String) -> Result<(& str, u16), LaunchServerError>
    {
        if let Some(game) = db.game_table.get(&game_id)
        {   
            let game_info = get_game_info(db, &game_id).unwrap();
            
            // TODO: Here, we should select a domain/public ip
            let address = "localhost";
            let port = get_free_port(db);
            
            //let program = "/home/defu/Projects/BlockBuster/build/src/server/Server";
            let program = exec_path;
            let map = "/home/defu/Projects/BlockBuster/resources/maps/Alpha2.bbm";
            let gamemode = "Deathmatch";

            let res = Command::new(program)
                .arg("-a").arg(address)
                .arg("-p").arg(port.to_string())
                .arg("-m").arg(map)
                .arg("-mp").arg(game.max_players.to_string())
                .arg("-sp").arg(game_info.players.to_string())
                .arg("-gm").arg(gamemode)
                .spawn();

            if let Err(_error) = res
            {
                return Err(LaunchServerError::CouldNotlaunch);
            }

            return Ok((address, port));
        }

        return Err(LaunchServerError::GameNotFound);
    }

    fn get_free_port(db : &database::DB) -> u16
    {
        let mut port = rand::thread_rng().gen_range(8000..8400);

        while is_port_in_use(db, port)
        {
            port = rand::thread_rng().gen_range(8000..8400);
        }

        return port;
    }

    fn is_port_in_use(db : &database::DB, port : u16) -> bool
    {
        for entry in db.game_table.get_all().into_iter()
        {
            if let Some(entry_port) = entry.port
            {
                if entry_port == port
                {
                    return true;
                }
            } 
        }

        return false;
    }

    fn notify_game_update(db : &database::DB, game_id : &uuid::Uuid) -> bool
    {
        let mut notified = false;
        if let Some(game_sem) = db.game_sem_table.get(&game_id)
        {
            game_sem.sem.notify_all();
            notified = true;
        }

        return notified;
    }

    fn get_game_details(db : &database::DB, game_id : &uuid::Uuid) -> Result<payload::response::GameDetails, QueryError>
    {
        let game_players = get_game_players_info(&db, &game_id);

        if let Ok(game_info) = get_game_info(db, &game_id){
            return Ok(payload::response::GameDetails{game_info, players :  game_players})
        }

        Err(QueryError::EntityNotFound)
    }

    fn is_game_full(db : &database::DB, game_id : &uuid::Uuid) -> Result<bool, QueryError>
    {
        let game_players = get_game_players_info(&db, &game_id);
        if let Some(game) = db.game_table.get(game_id)
        {
            return Ok(game_players.len() as u8 >= game.max_players);
        }

        return Err(QueryError::EntityNotFound);
    }

    fn is_game_empty(db : &database::DB, game_id : &uuid::Uuid) -> bool
    {
        let game_players = get_game_players_info(&db, &game_id);
        return game_players.is_empty();
    }

    #[derive(Debug)]
    enum QueryError
    {
        EntityNotFound
    }

    fn get_game_info(db : &database::DB, game_id : &uuid::Uuid) -> Result<payload::response::GameInfo, QueryError> 
    {
        if let Some(game) = db.game_table.get(game_id)
        {
            let players = get_game_players_info(&db, &game_id);
            let player_amount = players.len() as u8;
            let ping = 56;
            return Ok(payload::response::GameInfo{
                id : game.id,
                name : game.name, 
                map : game.map, 
                mode : game.mode, 
                max_players : game.max_players, 
                players : player_amount, 
                ping,
                chat : game.chat.to_vec(),

                address : game.address,
                port : game.port,
                state : game.state
            })
        }

        Err(QueryError::EntityNotFound)
    }

    fn get_game_players(db : &database::DB, game_id : &uuid::Uuid) -> Vec<entity::Player>
    {
        let mut game_players = Vec::new();
        for entry in db.player_game_table.get_all().into_iter()
        {
            let entry_game_id = entry.game_id;
            
            if *game_id == entry_game_id
            {
                let player = db.player_table.get(&entry.player_id).expect("Could not find player in playertable");
                game_players.push(player);
            }
        }

        game_players
    }

    fn get_game_players_info(db : &database::DB, game_id : &uuid::Uuid) -> Vec<payload::response::PlayerInfo>
    {
        let mut game_players = Vec::new();
        for entry in db.player_game_table.get_all().into_iter()
        {
            let entry_game_id = entry.game_id;
            
            if *game_id == entry_game_id
            {
                let player = db.player_table.get(&entry.player_id).expect("Could not find player in playertable");
                let ready = match entry.player_type {entity::PlayerType::Player(ready) => ready, _ => false};
                let host = match entry.player_type{entity::PlayerType::Host => true, _ => false};
                let player_info = payload::response::PlayerInfo{name : player.name, ready : ready, host : host};
                game_players.push(player_info);
            }
        }

        game_players
    }
}

pub mod filters
{
    use std::path::Path;

    use warp::Filter;
    use super::handlers;
    use crate::matchmaking::payload::request;
    use crate::matchmaking::database;

    pub fn get_routes(db : database::DB, exec_path : String) 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        login(db.clone())
        .or(list_games(db.clone()))
        .or(create_game(db.clone()))
        .or(join_game(db.clone()))
        .or(leave_game(db.clone()))
        .or(toggle_ready(db.clone()))
        .or(send_chat_msg(db.clone()))
        .or(update_game(db.clone()))
        .or(start_game(db.clone(), exec_path))
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

    pub fn send_chat_msg(db : database::DB) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        
        warp::post()
        .and(warp::path("send_chat_msg"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::SendChatMsg>())
        .and(filter.clone())
        .and_then(handlers::send_chat_msg)
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

    pub fn start_game(db : database::DB, exec_path : String)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        let param2 = warp::any().map(move || exec_path.clone());

        warp::post()
        .and(warp::path("start_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::StartGame>())
        .and(filter.clone())
        .and(param2.clone())
        .and_then(handlers::start_game)
    }
}