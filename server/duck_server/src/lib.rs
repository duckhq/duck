// Need macro_use for failure crate, but Rust doesn't think so...
#[allow(unused_imports)]
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;
extern crate log;

pub mod config;
mod engine;
mod utils;
mod web;

use failure::Error;
use log::info;

use config::ConfigurationLoader;

pub type DuckResult<T> = Result<T, Error>;

pub async fn run(config: impl ConfigurationLoader + 'static) -> DuckResult<()> {
    println!(r#"     ____             __  "#);
    println!(r#"    / __ \__  _______/ /__"#);
    println!(r#"   / / / / / / / ___/ //_/"#);
    println!(r#"  / /_/ / /_/ / /__/  <   "#);
    println!(r#" /_____/\____/\___/_/|_|  "#);
    println!();

    // Start engine.
    info!("Starting engine...");
    let engine_handle = engine::run(config)?;
    info!("Engine started.");

    // Start web server
    web::start().await?;

    // Stop the engine.
    engine_handle.stop()?;

    Ok(())
}
