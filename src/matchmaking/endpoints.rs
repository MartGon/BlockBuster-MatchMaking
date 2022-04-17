

pub mod handlers
{
    use std::fs::File;
    use std::io::Read;
    use std::io::Seek;
    use std::io::Write;
    use std::path::Path;
    use walkdir::{DirEntry};
    use yaml_rust::YamlEmitter;
    use yaml_rust::YamlLoader;
    use yaml_rust::yaml::Hash;
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
    use zip::write::FileOptions;

    // /login
    pub async fn login(player : payload::request::Login, db : database::DB) 
        -> Result<impl warp::Reply, warp::Rejection>{
        
        let player = entity::Player::new(player.username.clone());
        let player_copy = player.clone();
        db.player_table.insert(player.id, player);
        println!("Player login {}", player_copy.id);

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

    pub async fn create_game(cg_req : payload::request::CreateGame, db : database::DB, maps_folder : String)
    -> Result<impl warp::Reply, warp::Rejection>
    {
        let yml = read_map_yaml(&cg_req.map, &maps_folder);
        let version = yml["version"].as_str().unwrap().to_string();
            
        let game = entity::Game::new(cg_req.name, cg_req.map, version, cg_req.mode, cg_req.max_players);
        println!("Game key is {}", game.key);
        db.game_table.insert(game.id.clone(), game.clone());

        let game_sem = entity::GameSem::new(game.id);
        db.game_sem_table.insert(game.id, game_sem);

        join_game_as(&db, cg_req.player_id, game.id, true).unwrap();

        // Send response
        let response = get_game_details(&db, &game.id).unwrap();
        return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
    }

    pub async fn edit_game(eg_req : payload::request::EditGame, db : database::DB, maps_folder : String)
    -> Result<impl warp::Reply, warp::Rejection>
    {
        let player_id = eg_req.player_id;
        if let Some(player_game) = db.player_game_table.get(&player_id)
        {
            if let PlayerType::Host = player_game.player_type 
            {
                let yml = read_map_yaml(&eg_req.map, &maps_folder);
                let version = yml["version"].as_str().unwrap().to_string();
                
                if let Some(mut game) = db.game_table.get(&eg_req.game_id)
                {
                    game.name = eg_req.name; game.map = eg_req.map; game.mode = eg_req.mode; game.map_version = version;
                    println!("Game key is {}", game.key);
                    db.game_table.insert(game.id.clone(), game.clone());

                    let game_sem = entity::GameSem::new(game.id);
                    db.game_sem_table.insert(game.id, game_sem);
                    
                    get_game_players(&db, &eg_req.game_id).into_iter().for_each(|x| set_player_ready(&db, &x.id, false));
                    notify_game_update(&db, &eg_req.game_id);

                    // Send response
                    let response = get_game_details(&db, &game.id).unwrap();
                    return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
                }
                let err = format!("Could not find game with id {}", eg_req.game_id.to_string());
                return Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
            }
            let err = format!("Player was not host");
            return Ok(warp::reply::with_status(reply::json(&err), StatusCode::FORBIDDEN));
        }
        let err = format!("Could not find player with id {}", eg_req.game_id.to_string());
        Ok(warp::reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
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
        
        leave_game_fn(&db, player_id);

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
            
            if !update_game_req.forced
            {
                game_sem.sem.wait_timeout(game_sem.mutex.lock().unwrap(), Duration::from_secs(15)).unwrap();
            }

            if let Ok(response) = get_game_details(&db, &game.id)
            {
                return Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK));
            }
        }

        let err = format!("Could not find game with id {}", game_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }

    pub async fn start_game(start_game_req : payload::request::StartGame, db : database::DB, exec_path : String, maps_folder : String) -> Result<impl warp::Reply, Infallible>
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
                        if let Ok((address, port)) = launch_game(&db, game_id, exec_path, maps_folder)
                        {
                            game.port = Some(port);
    
                            game.address = Some(String::from(address));
                            game.state = GameState::InGame;

                            db.game_table.insert(game.id, game);
                            get_game_players(&db, &game_id).into_iter().for_each(|x| set_player_ready(&db, &x.id, false));
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

    pub async fn get_available_maps(maps_folder : String) -> Result<impl warp::Reply, warp::Rejection>
    {
        let maps : Vec<String>;
        let maps_folder_path = Path::new(&maps_folder);
        let paths = maps_folder_path.read_dir().unwrap();
        maps = paths.into_iter()
        .filter(|r| r.is_ok()) 
        .map(|r| r.unwrap().path()) 
        .filter(|r| r.is_dir()) 
        .map(|r| r.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
        
        let mut maps_info = Vec::<payload::response::MapInfo>::new();
        for map in maps
        {
            let yml = read_map_yaml(&map, &maps_folder);
            let game_modes = yml["gamemodes"].as_vec().unwrap().into_iter().map(|x| x.as_str().unwrap().to_string()).collect();
            if let Some(version) = yml["version"].as_str()
            {
                let map_info = payload::response::MapInfo{ map_name : map, supported_gamemodes : game_modes, map_version : version.to_string() };
                maps_info.push(map_info);
            }            
        }

        let response = payload::response::AvailableMaps{maps : maps_info};
        Ok(warp::reply::json(&response))
    }

    pub async fn download_map(download_map_req : payload::request::DownloadMap, maps_folder : String) -> Result<impl warp::Reply, warp::Rejection>
    {
        let map = download_map_req.map_name; 
        
        // Check filename. Shouldn't contain slahes. can be a security issue
        if !is_map_name_valid(&map)
        {
            let err = format!("Illegal character in map name");
            return Ok(reply::with_status(reply::json(&err), StatusCode::FORBIDDEN));
        }

        let map_file_name = map.clone() + ".zip";
        let maps_folder_path = Path::new(&maps_folder);
        let map_folder = get_map_folder(&map, &maps_folder);
        let map_folder = Path::new(&map_folder);
        //println!("Map path folder is {}", map_folder.to_str().unwrap());

        if map_folder.exists() && map_folder.is_dir()
        {
            let zip_path = maps_folder_path.join(map_file_name);

            // Create Zip File - This is no longer needed
            /*
            let file = File::create(&zip_path).unwrap();
            let walkdir = WalkDir::new(map_path.to_str().unwrap());
            let it = walkdir.into_iter();
            let res = zip_dir(&mut it.filter_map(|e| e.ok()), map_path.to_str().unwrap(),
                 file, zip::CompressionMethod::Stored);
            */

            // Read file, encode and write response
            let mut buffer = Vec::new();
            let mut file = File::open(zip_path).unwrap();
            file.read_to_end(&mut buffer).unwrap();
            let output = base64::encode(buffer);

            let response = payload::response::DownloadMap{map : output};
            return Ok(reply::with_status(reply::json(&response), StatusCode::OK));
        }

        let err = format!("Could not find map with name {}", map);
        return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
    }

    pub async fn get_map_picture(map_picture_req : payload::request::MapPicture, maps_folder : String) -> Result<impl warp::Reply, Infallible>
    {
        let map = map_picture_req.map_name; 
        
        // Check filename. Shouldn't contain slahes. can be a security issue
        if !is_map_name_valid(&map)
        {
            let err = format!("Illegal character in map name");
            return Ok(reply::with_status(reply::json(&err), StatusCode::FORBIDDEN));
        }

        let pic_file_name =  map.clone() + ".jpg";
        let map_folder = get_map_folder(&map, &maps_folder);
        let map_folder = Path::new(&map_folder);

        if map_folder.exists() && map_folder.is_dir()
        {
            let pic_path = map_folder.join(pic_file_name);
            if let Ok(mut pic_file) = std::fs::File::open(pic_path)
            {
                let mut buffer = Vec::new();
                pic_file.read_to_end(&mut buffer).unwrap();
                
                let b64 = base64::encode(buffer);
                let response = payload::response::MapPicture{map_picture : b64};
                return Ok(reply::with_status(reply::json(&response), StatusCode::OK));
            }
            else
            {
                let err = format!("Could not find map picture");
                return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
            }
        }

        let err = format!("Could not find map with name {}", map);
        return Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND));
    }

    pub async fn upload_map(upload_map_req : payload::request::UploadMap, maps_folder : String) -> Result<impl warp::Reply, warp::Rejection>
    {
        // Check filename. Shouldn't contain slahes. can be a security issue
        let map = upload_map_req.map_name; 
        if !is_map_name_valid(&map)
        {
            let err = format!("Illegal character in map name");
            return Ok(reply::with_status(reply::json(&err), StatusCode::FORBIDDEN));
        }

        // Check if it's an update
        let map_filename = map.clone() + ".zip";
        let map_folder_path = maps_folder;
        let maps_folder = Path::new(&map_folder_path);
        let map_folder = maps_folder.join(&map);

        let yml_path = map_folder.join(map.clone() + ".yml");
        if map_folder.exists() && map_folder.is_dir()
        {
            let yml = read_map_yaml(&map, &map_folder_path);
            let pass = yml["password"].as_str().unwrap();

            if pass != upload_map_req.password
            {
                let err = format!("Password to update map {} was not correct", map);
                return Ok(reply::with_status(reply::json(&err), StatusCode::FORBIDDEN));
            }
        }

        // Write map
        let zip_path = maps_folder.join(map_filename);
        //println!("Zip file path is {}", zip_path.to_str().unwrap());
        if let Ok(buffer) = base64::decode(upload_map_req.map_zip)
        {
            // Create dir
            if !map_folder.exists()
            {
                std::fs::create_dir_all(&map_folder).unwrap();
            }
            
            // Write zip file
            {
                let mut file = File::create(&zip_path).unwrap();
                file.write_all(&buffer).unwrap();
            }
            
            // Extract zip
            let file = std::fs::File::open(&zip_path).unwrap();
            zip_extract(file, &maps_folder.to_str().unwrap().to_string());

            // Create config file
            let mut output = String::new();
            let mut hash = Hash::new();
            hash.insert(yaml_rust::Yaml::String("password".to_string()), yaml_rust::Yaml::String(upload_map_req.password));
            hash.insert(yaml_rust::Yaml::String("version".to_string()), yaml_rust::Yaml::String(upload_map_req.map_version));

            let gamemodes = upload_map_req.supported_gamemodes.iter().map(|x| yaml_rust::Yaml::String(x.to_string())).collect();
            hash.insert(yaml_rust::Yaml::String("gamemodes".to_string()), yaml_rust::Yaml::Array(gamemodes));
            let mut emmiter = YamlEmitter::new(&mut output);
            emmiter.dump(&yaml_rust::Yaml::Hash(hash)).unwrap();

            {
                let mut file = File::create(&yml_path).unwrap();
                file.write(output.as_bytes()).unwrap();
            }
        }

        let response = "Success";
        return Ok(reply::with_status(reply::json(&response), StatusCode::OK));
    }

    pub async fn notify_server_event(server_event : payload::request::NotifyServerEvent, db : database::DB) -> Result<impl warp::Reply, Infallible>
    {
        use payload::request::ServerEvent;
        let event_type = server_event.event;

        let game_id = server_event.game_id;
        if let Some(game) = db.game_table.get(&server_event.game_id)
        {
            if game.key == server_event.server_key
            {
                match event_type
                {
                    ServerEvent::PlayerLeft{player_id} => {
                        println!("Player with id {} left game {}", player_id, game_id);
                        leave_game_fn(&db, player_id);
                    },
                    ServerEvent::GameEnded => { 
                        println!("Game {} is over", game_id);
                        set_game_state(&db, &game_id, GameState::InLobby);
                    },
                }

                return Ok(reply::with_status(reply::json(&"".to_string()), StatusCode::OK));
            }

            let err = format!("Key was not correct for game with id {}", server_event.game_id.to_string());
            return Ok(reply::with_status(reply::json(&err), StatusCode::FORBIDDEN));
        }

        let err = format!("Could not find game with id {}", server_event.game_id.to_string());
        Ok(reply::with_status(reply::json(&err), StatusCode::NOT_FOUND))
    }


    // Helper Functions

    fn read_map_yaml(map : &String, maps_folder : &String) -> yaml_rust::Yaml
    {
        let map_folder = get_map_folder(map, maps_folder);
        let map_folder = Path::new(&map_folder);
        let yml_path = map_folder.join(map.clone() + ".yml");
        println!("YML path is {}", yml_path.to_str().unwrap());

        let mut yml_file = std::fs::File::open(&yml_path).unwrap();
        let mut data_str = String::new();
        yml_file.read_to_string(&mut data_str).unwrap();
        let yml = YamlLoader::load_from_str(&data_str).unwrap().remove(0);

        return yml;
    }

    fn get_map_folder(map : &String, maps_folder : &String) -> String
    {
        let maps_folder = Path::new(&maps_folder);
        let map_folder = maps_folder.join(&map);
        return map_folder.as_os_str().to_str().unwrap().to_string();
    }

    fn zip_extract(file : File, folder : &String)
    {
        let folder = Path::new(folder.as_str());
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = folder.join(file.mangled_name());
    
            if (&*file.name()).ends_with('/') {
                //println!("File {} extracted to \"{}\"", i, outpath.as_path().display());
                std::fs::create_dir_all(&outpath).unwrap();
            } else {
                //println!("File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p).unwrap();
                    }
                }
                let mut outfile = std::fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
    
            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
    
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
                }
            }
        }
    }

    fn _zip_dir<T>(it: &mut dyn Iterator<Item=DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod)
              -> zip::result::ZipResult<()> where T: Write+Seek
    {
        let mut zip = zip::ZipWriter::new(writer);
        let options = FileOptions::default().compression_method(method).unix_permissions(0o755);
        
        for entry in it {
            let path = entry.path();
            let name = path.strip_prefix(Path::new(prefix)).unwrap();

            if path.is_file() {
                zip.start_file(name.to_str().unwrap(), options)?;
                let mut f = File::open(path)?;
                
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&*buffer)?;

                buffer.clear();
            } else if name.as_os_str().len() != 0 {

                zip.add_directory(name.to_str().unwrap(), options)?;
            }
        }

        zip.finish()?;
        Result::Ok(())
    }

    fn is_map_name_valid(map_name : &String) -> bool
    {
        let reg = regex::Regex::new(r"[A-z0-9]*").unwrap();
        return reg.is_match(map_name);
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

    fn leave_game_fn(db : &database::DB, player_id : uuid::Uuid)
    {
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
    }

    fn launch_game(db : &database::DB, game_id : uuid::Uuid, exec_path : String, maps_folder : String) -> Result<(& str, u16), LaunchServerError>
    {
        if let Some(game) = db.game_table.get(&game_id)
        {   
            let game_info = get_game_info(db, &game_id).unwrap();
            
            // TODO: Here, we should select a domain/public ip
            let address = "localhost";
            let port = get_free_port(db);
            
            let program = exec_path;
            let maps_folder = Path::new(&maps_folder);
            let map_folder = maps_folder.join(&game_info.map);
            let map_path = map_folder.join(game_info.map + ".bbm");
            let gamemode = game_info.mode;

            let res = Command::new(program)
                .arg("-a").arg(address)
                .arg("-p").arg(port.to_string())
                .arg("-m").arg(map_path)
                .arg("-mp").arg(game.max_players.to_string())
                .arg("-sp").arg(game_info.players.to_string())
                .arg("-gm").arg(gamemode)
                
                .arg("-mmid").arg(game_id.to_string())
                .arg("-mmk").arg(game.key.to_string())
                .spawn();

            if let Err(_error) = res
            {
                return Err(LaunchServerError::CouldNotlaunch);
            }

            set_game_state(&db, &game_id, GameState::InGame);

            return Ok((address, port));
        }

        return Err(LaunchServerError::GameNotFound);
    }

    fn set_game_state(db : &database::DB, game_id : &uuid::Uuid, state : GameState)
    {
        if let Some(mut game) = db.game_table.get(&game_id)
        {
            game.state = state;
            db.game_table.insert(game.id, game);
        }
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
            
            // Update last time this game had changes
            if let Some(mut game) = db.game_table.get(&game_id)
            {
                game.last_update = std::time::SystemTime::now();
                db.game_table.insert(game.id, game);
            }
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

    fn _is_game_empty(db : &database::DB, game_id : &uuid::Uuid) -> bool
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
                map_version : game.map_version,
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

    fn set_player_ready(db : &database::DB, player_id : &uuid::Uuid, ready : bool)
    {
        if let Some(mut player_game)  = db.player_game_table.get(&player_id)
        {
            if let entity::PlayerType::Player(_is_ready) = player_game.player_type
            {
                player_game.player_type = entity::PlayerType::Player(ready);
                notify_game_update(&db, &player_game.game_id);
                db.player_game_table.insert(player_id.clone(), player_game);
            }
        }
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
    use warp::Filter;
    use super::handlers;
    use crate::matchmaking::payload::request;
    use crate::matchmaking::database;

    pub fn get_routes(db : database::DB, exec_path : String, maps_folder : String) 
        -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        login(db.clone())
        .or(list_games(db.clone()))
        .or(create_game(db.clone(), maps_folder.clone()))
        .or(edit_game(db.clone(), maps_folder.clone()))
        .or(join_game(db.clone()))
        .or(leave_game(db.clone()))
        .or(toggle_ready(db.clone()))
        .or(send_chat_msg(db.clone()))
        .or(update_game(db.clone()))
        .or(start_game(db.clone(), exec_path, maps_folder.clone()))
        .or(get_available_maps(maps_folder.clone()))
        .or(download_map(maps_folder.clone()))
        .or(get_map_picture(maps_folder.clone()))
        .or(upload_map(maps_folder.clone()))
        .or(notify_server_event(db.clone()))
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

    pub fn create_game(db : database::DB, maps_folder : String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        let param3 = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("create_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::CreateGame>())
        .and(filter.clone())
        .and(param3.clone())
        .and_then(handlers::create_game)
    }

    pub fn edit_game(db : database::DB, maps_folder : String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        let param3 = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("edit_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::EditGame>())
        .and(filter.clone())
        .and(param3.clone())
        .and_then(handlers::edit_game)
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

    pub fn start_game(db : database::DB, exec_path : String, maps_folder : String)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());
        let param2 = warp::any().map(move || exec_path.clone());
        let param3 = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("start_game"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<request::StartGame>())
        .and(filter.clone())
        .and(param2.clone())
        .and(param3.clone())
        .and_then(handlers::start_game)
    }

    pub fn get_available_maps(maps_folder : String)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("get_available_maps"))
        .and(warp::path::end())
        .and(filter.clone())
        .and_then(handlers::get_available_maps)
    }

    pub fn download_map(maps_folder : String)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("download_map"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json::<request::DownloadMap>())
        .and(filter.clone())
        .and_then(handlers::download_map)
    }

    pub fn get_map_picture(maps_folder : String)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let map_folder = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("get_map_picture"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::json::<request::MapPicture>())
        .and(map_folder)
        .and_then(handlers::get_map_picture)
    }

    pub fn upload_map(maps_folder : String)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || maps_folder.clone());

        warp::post()
        .and(warp::path("upload_map"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 1024 * 16)) // 16 MB
        .and(warp::body::json::<request::UploadMap>())
        .and(filter.clone())
        .and_then(handlers::upload_map)
    }

    pub fn notify_server_event(db : database::DB)
    -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        let filter = warp::any().map(move || db.clone());

        warp::post()
        .and(warp::path("notify_server_event"))
        .and(warp::path::end())
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::json::<request::NotifyServerEvent>())
        .and(filter.clone())
        .and_then(handlers::notify_server_event)
    }
}