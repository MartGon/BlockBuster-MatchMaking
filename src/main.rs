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

    // Public address
    #[clap(short, long)]
    game_address: String,

    // Port to listen on
    #[clap(short, long)]
    port: u16,

    // Server executable plath
    #[clap(short, long)]
    server_path: String,

    // Maps folder
    #[clap(short, long, default_value = "./maps")]
    maps_folder : String,

    // Serve tick rate
    #[clap(short, long, default_value = "0.020")]
    tick_rate : f32,
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
        println!("Maps folder is invalid: {}", args.maps_folder);
        std::process::exit(-1);
    }
    println!("Maps folder is: {}", args.maps_folder);

    let address = format!("{}:{}", args.address, args.port);
    let address = ToSocketAddrs::to_socket_addrs(&address).expect("Couldn't parse socket address").next().unwrap();
    let game_address = format!("{}:{}", args.game_address, args.port);
    ToSocketAddrs::to_socket_addrs(&game_address).expect("Couldn't parse game address").next().unwrap();

    let db = database::DB::new();

    let copy = db.clone();
    std::thread::spawn(move ||{
        update(&copy);
    });
    
    let routes = endpoints::filters::get_routes(db, args.server_path, args.maps_folder, args.game_address, args.tick_rate, args.port);
    warp::serve(routes).run((address.ip(), args.port)).await;
}


fn update(db : &database::DB)
{
    let SLEEP_DURATION  = std::time::Duration::from_secs(5);
    let MAX_DURATION = std::time::Duration::from_secs(60 * 3); // 3 MIN
    loop {

        let now = std::time::SystemTime::now();

        let games = db.game_table.get_all();
        games.into_iter().for_each(|game| {
            let elapsed = now.duration_since(game.last_update);
            let elapsed = elapsed.unwrap();
            if elapsed > MAX_DURATION && matches!(game.state, GameState::InLobby)
            {
                db.game_table.remove(&game.id);
                println!("Removing AFK game with id {}", game.id);
            }
        });

        std::thread::sleep(SLEEP_DURATION);
    }
}