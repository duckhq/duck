#[macro_use]
extern crate serde;
extern crate log;

pub mod config;
mod web;

use failure::Error;

use config::ConfigurationLoader;

pub type DuckResult<T> = Result<T, Error>;

pub async fn run(_config: impl ConfigurationLoader) -> DuckResult<()> {
    log::info!("Running Duck!");

    // Start web server
    web::start().await?;

    Ok(())
}
