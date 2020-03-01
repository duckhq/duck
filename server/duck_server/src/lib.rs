#[macro_use]
extern crate serde;
extern crate log;

pub mod config;

use failure::Error;

use config::ConfigurationHandle;

pub type DuckResult<T> = Result<T, Error>;

pub fn run(_config: impl ConfigurationHandle) -> DuckResult<()> {
    log::info!("Running Duck!");
    Ok(())
}
