extern crate env_logger;
extern crate log;

use std::path::PathBuf;
use std::process::exit;

use env_logger::Env;
use log::error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "duck")]
struct Opt {
    /// The configuration file to use
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "config.json",
        env = "DUCK_CONFIG"
    )]
    config: PathBuf,
    /// The server address to bind to
    #[structopt(name = "bind", short, long, env = "DUCK_BIND")]
    server_address: Option<String>,
    /// Show verbose output
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(subcommand)]
    commands: Option<Command>
}

#[derive(StructOpt)]
enum Command {
    /// Generates the JSON schema
    Schema
}

fn main() {
    let args = Opt::from_args();

    // Was a sub command invoked?
    if let Some(command) = &args.commands {
        match command {
            Command::Schema => { 
                println!("{}", duck::get_schema());
            }
        }
        exit(0);
    };

    initialize_logging(&args);

    match duck::run(args.config, args.server_address) {
        Result::Ok(_) => exit(0),
        Result::Err(e) => {
            error!("An error occured: {}", e);
            exit(-1);
        }
    };
}

fn initialize_logging(args: &Opt) {
    let level = if args.verbose { "debug" } else { "info" };
    let filter = format!(
        "{},actix=off,mio=off,tokio=off,want=off,hyper=off,reqwest=off",
        level
    );

    env_logger::init_from_env(
        Env::default()
            .filter_or("DUCK_LOG_LEVEL", filter)
            .write_style_or("DUCK_LOG_STYLE", "always"),
    );
}
