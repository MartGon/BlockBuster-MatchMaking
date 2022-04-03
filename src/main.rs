mod matchmaking;

use matchmaking::endpoints;
use matchmaking::entity;
use matchmaking::database;

use clap::Parser;
use matchmaking::entity::GameState;

use std::net::ToSocketAddrs;
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
    // Address to listen on
    #[clap(short, long)]
    address: String,

    // Port to listen on
    #[clap(short, long)]
    port: u16,

    // Server executable plath
    #[clap(short, long)]
    server_path: String,

    // Maps folder
    #[clap(short, long, default_value = "./maps")]
    maps_folder : String
}


#[tokio::main]
async fn main() {

    let args = Args::parse();

    let server_path = Path::new(&args.server_path);
    if !server_path.exists()
    {
        println!("Path to server exectuble is invalid: {}", args.server_path);
        std::process::exit(-1);
    }
    let maps_folder = Path::new(&args.maps_folder);
    if !maps_folder.exists() && !maps_folder.is_dir()
    {
        println!("Maps folder is invalid: {}", args.server_path);
        std::process::exit(-1);
    }
    println!("Maps folder is: {}", args.maps_folder);

    let address = format!("{}:{}", args.address, args.port);
    let address = ToSocketAddrs::to_socket_addrs(&address).expect("Couldn't parse socket address").next().unwrap();

    let db = database::DB::new();

    let name = "Sample".to_string();
    let map = "Kobra".to_string();
    let mode = "DeathMatch".to_string();
    let max_players : u8 = 16;
    let sample_game = matchmaking::entity::Game::new(name, map, mode, max_players);
    let game_sem = entity::GameSem::new(sample_game.id.clone());
    db.game_table.insert(sample_game.id.clone(), sample_game.clone());
    db.game_sem_table.insert(sample_game.id, game_sem);

    let copy = db.clone();
    std::thread::spawn(move ||{
        update(&copy);
    });
    
    let routes = endpoints::filters::get_routes(db, args.server_path, args.maps_folder);
    warp::serve(routes).run((address.ip(), args.port)).await;

    // TODO: Check the database periodically for AFK games
}


fn update(db : &database::DB)
{
    let SLEEP_DURATION  = std::time::Duration::from_secs(5);
    let MAX_DURATION = std::time::Duration::from_secs(60 * 1); // 3 MIN
    loop {

        let now = std::time::SystemTime::now();

        let games = db.game_table.get_all();
        games.into_iter().for_each(|game| {
            println!("Found game with id {}", game.id);
            let elapsed = now.duration_since(game.last_update);
            let elapsed = elapsed.unwrap();
            if elapsed > MAX_DURATION && matches!(game.state, GameState::InLobby)
            {
                db.game_table.remove(&game.id);
                println!("Removing game with id {}", game.id);
            }
        });

        std::thread::sleep(SLEEP_DURATION);
    }
}