extern crate log;

use env_logger::Env;
use structopt::StructOpt;
use commands::run;

mod commands;
mod config;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long, parse(from_str = parse_level), env = "DUCK_LEVEL")]
    level: Option<LogLevel>,
    /// Available subcommands
    #[structopt(subcommand)]
    command: Command,
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
    Run(run::Arguments)
}

#[async_std::main]
async fn main() {
    let args = Args::from_args();

    initialize_logging(args.level);

    let result = match args.command {
        Command::Run(args) => {
            commands::run::execute(&args)
        }
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("An error occured: {}", e);
            std::process::exit(-1);
        },
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