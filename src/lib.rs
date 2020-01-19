#![allow(clippy::needless_return)]
#![allow(clippy::mutex_atomic)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

mod api;
mod builds;
mod collectors;
mod config;
mod engine;
mod observers;
mod utils;

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::Configuration;
use crate::engine::state::EngineState;
use crate::utils::DuckResult;

pub fn run<T: Into<PathBuf>>(config_path: T, server_address: Option<String>) -> DuckResult<()> {
    // Load and validate the configuration file.
    let config = Configuration::from_file(config_path.into())?;

    // Start the engine.
    let state = Arc::new(EngineState::new());
    let engine_handle = engine::run(&config, &state);

    // Start the HTTP server.
    // This will block until CTRL+C is pressed.
    api::start_and_block(state, server_address)?;

    // Stop the engine.
    engine_handle.stop()?;
    Ok(())
}
