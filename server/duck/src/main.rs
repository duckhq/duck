extern crate log;

use env_logger::Env;
use structopt::StructOpt;

mod commands;
mod config;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long, parse(from_str = parse_level), env = "DUCK_LEVEL")]
    level: Option<LogLevel>,
    /// Available subcommands
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Debug)]
enum LogLevel {
    Info,
    Debug,
    Trace,
}

fn parse_level(src: &str) -> LogLevel {
    match src {
        "debug" => LogLevel::Debug,
        "trace" => LogLevel::Trace,
        _ => LogLevel::Info,
    }
}

#[derive(StructOpt)]
enum Command {
    /// Starts the Duck server
    Start(commands::start::Arguments),
}

#[actix_rt::main]
async fn main() {
    let args = Args::from_args();
    initialize_logging(args.level);

    // Get the command (default to 'run')
    let command = match args.command {
        None => Command::Start(commands::start::Arguments::default()),
        Some(command) => command,
    };

    // Execute the command
    let result = match command {
        Command::Start(args) => commands::start::execute(args).await,
    };

    // Return the correct exit code
    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("An error occured: {}", e);
            std::process::exit(-1);
        }
    }
}

fn initialize_logging(level: Option<LogLevel>) {
    let level = match level {
        None => "info",
        Some(level) => match level {
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        },
    };

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
