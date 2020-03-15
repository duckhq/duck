use std::path::PathBuf;

use structopt::StructOpt;

use super::start::DEFAULT_CONFIG;
use duck::DuckResult;

///////////////////////////////////////////////////////////
// Arguments

#[derive(StructOpt, Debug)]
pub struct Arguments {
    /// The configuration file
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = DEFAULT_CONFIG,
        env = "DUCK_CONFIG"
    )]
    pub config: PathBuf,
}

///////////////////////////////////////////////////////////
// Command

pub async fn execute(args: Arguments) -> DuckResult<()> {
    duck::validate_config(args.config).await?;
    Ok(())
}
