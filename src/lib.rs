#![allow(clippy::needless_return)]
#![allow(clippy::mutex_atomic)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

use std::path::PathBuf;

use crate::config::Configuration;
use crate::utils::DuckResult;

mod api;
mod builds;
mod config;
mod engine;
mod providers;
mod utils;

pub fn run<T: Into<PathBuf>>(config_path: T, server_address: Option<String>) -> DuckResult<()> {
    // Load and validate the configuration file.
    let config = Configuration::from_file(config_path.into())?;

    // Start the engine.
    let engine = engine::Engine::new(&config)?;
    let engine_handle = engine.run()?;

    // Start the HTTP server.
    // This will block until CTRL+C is pressed.
    api::start_and_block(engine.get_state(), server_address)?;

    // Stop the engine.
    engine_handle.stop()?;
    Ok(())
}
