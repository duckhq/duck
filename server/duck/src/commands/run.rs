use std::path::PathBuf;
use structopt::StructOpt;

use duck_server::DuckResult;
use crate::config::FileConfiguration;

#[derive(StructOpt, Debug)]
pub(crate) struct Arguments {
    /// The configuration file
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "config.json",
        env = "DUCK_CONFIG"
    )]
    config: PathBuf,
}

/// Executes the run command.
pub(crate) fn execute(args: &Arguments) -> DuckResult<()> {
    log::info!("{:?}", args);
    duck_server::run(FileConfiguration { })
}
