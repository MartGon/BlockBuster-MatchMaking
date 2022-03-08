mod matchmaking;

use matchmaking::endpoints;
use matchmaking::entity;
use matchmaking::database;

use clap::Parser;

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
    server_path: String
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
    
    let routes = endpoints::filters::get_routes(db, args.server_path);
    warp::serve(routes).run((address.ip(), args.port)).await;
}
