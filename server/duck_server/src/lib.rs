pub mod config;

use failure::Error;

use config::ConfigurationHandle;

pub type DuckResult<T> = Result<T, Error>;

pub fn run(_config: impl ConfigurationHandle) -> DuckResult<()> {
    println!("Running Duck!");
    Ok(())
}