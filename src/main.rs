extern crate env_logger;
extern crate log;

use duck::DuckResult;
use env_logger::Env;
use log::error;
use structopt::StructOpt;

mod commands;

#[derive(StructOpt)]
#[structopt(name = "Duck")]
struct Opt {
    /// The log level to use (info, debug, trace)
    #[structopt(short, long, parse(from_str = parse_level), env = "DUCK_LEVEL")]
    level: Option<LogLevel>,
    /// Disables the startup banner
    #[structopt(short, long)]
    no_logo: bool,
    /// Available subcommands
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug)]
enum LogLevel {
    Information,
    Debug,
    Trace,
}

fn parse_level(src: &str) -> LogLevel {
    match src {
        "debug" => LogLevel::Debug,
        "trace" => LogLevel::Trace,
        _ => LogLevel::Information,
    }
}

#[derive(StructOpt)]
enum Command {
    /// Starts the Duck server
    Start(commands::start::Arguments),
    /// Generates the JSON schema
    Schema(commands::schema::Arguments),
}

impl Command {
    pub fn show_logo(&self) -> bool {
        match self {
            Command::Start(_) => true,
            Command::Schema(_) => false,
        }
    }
}

#[actix_rt::main]
async fn main() {
    let args = Opt::from_args();
    initialize_logging(&args.level);

    if args.command.show_logo() && !args.no_logo {
        println!(r#"     ____             __  "#);
        println!(r#"    / __ \__  _______/ /__"#);
        println!(r#"   / / / / / / / ___/ //_/"#);
        println!(r#"  / /_/ / /_/ / /__/  <   "#);
        println!(r#" /_____/\____/\___/_/|_|  "#);
        println!();
    }

    // Execute the command
    let result = match args.command {
        Command::Start(args) => commands::start::execute(args).await,
        Command::Schema(args) => commands::schema::execute(args).await,
    };

    // Return the correct exit code
    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            error!("An error occured: {}", e);
            std::process::exit(-1);
        }
    }
}

fn initialize_logging(level: &Option<LogLevel>) {
    let level = match level {
        None => "info",
        Some(level) => match level {
            LogLevel::Information => "info",
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
