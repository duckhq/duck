#[macro_use]
extern crate serde;
extern crate log;

pub mod config;
mod web;

use failure::Error;

use config::ConfigurationHandle;

pub type DuckResult<T> = Result<T, Error>;

pub async fn run(_config: impl ConfigurationHandle) -> DuckResult<()> {
    log::info!("Running Duck!");

    // Start web server
    web::start().await?;

    Ok(())
}
