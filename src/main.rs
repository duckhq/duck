extern crate log;

use log::error;
use simplelog::*;
use structopt::StructOpt;

use duck::DuckResult;

mod commands;

#[derive(StructOpt)]
#[structopt(name = "Duck")]
struct Opt {
    /// The log level to use (info, debug, trace)
    #[structopt(short, long, parse(from_str = parse_level), env = "DUCK_LEVEL")]
    level: Option<LogLevel>,
    /// Whether or not to log to file.
    #[structopt(long = "file", short = "f")]
    log_to_file: bool,
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
    /// Validates the Duck configuration
    Validate(commands::validate::Arguments),
    /// Starts Duck as a Windows service
    #[cfg(windows)]
    #[structopt(setting = structopt::clap::AppSettings::Hidden)]
    Service,
    /// Installs Duck as a Windows service.
    /// Requires administrator user.
    #[cfg(windows)]
    Install,
    /// Uninstalls the Duck Windows service.
    /// Requires administrator user.
    #[cfg(windows)]
    Uninstall,
}

impl Command {
    pub fn show_logo(&self) -> bool {
        match self {
            Command::Start(_) => true,
            Command::Schema(_) => false,
            Command::Validate(_) => false,
            #[cfg(windows)]
            Command::Service => false,
            #[cfg(windows)]
            Command::Install => false,
            #[cfg(windows)]
            Command::Uninstall => false,
        }
    }
}

#[actix_rt::main]
async fn main() {
    let args = Opt::from_args();

    initialize_logging(&args.level, args.log_to_file)
        .expect("An error occured while setting up logging");

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
        Command::Schema(args) => commands::schema::execute(args),
        Command::Validate(args) => commands::validate::execute(args),
        #[cfg(windows)]
        Command::Service => commands::service::start(),
        #[cfg(windows)]
        Command::Install => commands::service::install(),
        #[cfg(windows)]
        Command::Uninstall => commands::service::uninstall(),
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

fn initialize_logging(level: &Option<LogLevel>, log_to_file: bool) -> DuckResult<()> {
    let level = match level {
        None => LevelFilter::Info,
        Some(level) => match level {
            LogLevel::Information => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        },
    };

    let padding = match level {
        LevelFilter::Info => LevelPadding::Off,
        _ => LevelPadding::Left,
    };

    let mut config = ConfigBuilder::new();
    config.set_level_padding(padding);
    config.add_filter_ignore_str("actix");
    config.add_filter_ignore_str("mio");
    config.add_filter_ignore_str("tokio");
    config.add_filter_ignore_str("want");
    config.add_filter_ignore_str("hyper");
    config.add_filter_ignore_str("reqwest");
    config.add_filter_ignore_str("rustls");
    config.add_filter_ignore_str("h2");
    let config = config.build();

    if log_to_file {
        // Log both to file
        let file = std::fs::File::create(std::env::current_exe()?.with_file_name("duck.log"))?;
        CombinedLogger::init(vec![WriteLogger::new(level, config, file)])?;
    } else {
        // Log to stdout
        let logger = match TermLogger::new(level, config.clone(), TerminalMode::Mixed) {
            Some(logger) => logger as Box<dyn SharedLogger>,
            None => SimpleLogger::new(level, config) as Box<dyn SharedLogger>,
        };
        CombinedLogger::init(vec![logger])?;
    }

    Ok(())
}
