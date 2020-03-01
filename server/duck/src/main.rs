use structopt::StructOpt;
use commands::run;

mod commands;
mod config;

#[derive(StructOpt)]
struct Args {
    /// Available subcommands
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    Run(run::Arguments)
}

#[async_std::main]
async fn main() {
    let args = Args::from_args();
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
