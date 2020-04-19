#![allow(clippy::needless_return)]
#![allow(clippy::mutex_atomic)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate derive_builder;

use std::path::PathBuf;

use failure::Error;
use log::info;

use crate::config::loader::JsonConfigurationLoader;
use crate::config::ConfigurationLoader;
use crate::utils::text::EnvironmentVariableProvider;

pub type DuckResult<T> = Result<T, Error>;

mod api;
mod builds;
mod config;
mod engine;
mod filters;
mod providers;
mod utils;

#[allow(dead_code)]
mod query;

///////////////////////////////////////////////////////////
// Handle

pub struct DuckHandle {
    engine: engine::EngineHandle,
    http: api::HttpServerHandle,
}

impl DuckHandle {
    pub async fn stop(self) -> DuckResult<()> {
        self.engine.stop()?;
        self.http.stop().await;
        Ok(())
    }
}

///////////////////////////////////////////////////////////
// Run

pub fn run<T: Into<PathBuf>>(
    config_path: T,
    server_address: Option<String>,
) -> DuckResult<DuckHandle> {
    // Write some info to the console.
    info!("Version {}", utils::VERSION);

    // Start the engine.
    let loader = JsonConfigurationLoader::new(config_path.into());
    let engine = engine::Engine::new()?;
    let engine_handle = engine.run(loader)?;

    // Start the HTTP server.
    let server = api::start(engine.get_state(), server_address)?;

    Ok(DuckHandle {
        engine: engine_handle,
        http: server,
    })
}

///////////////////////////////////////////////////////////
// Schema

pub fn get_schema() -> String {
    let settings = schemars::gen::SchemaSettings::draft07().with(|s| {
        s.option_nullable = false;
        s.option_add_null_type = false;
    });
    let gen = settings.into_generator();
    let schema = gen.into_root_schema_for::<config::Configuration>();
    serde_json::to_string_pretty(&schema).unwrap()
}

///////////////////////////////////////////////////////////
// Validate

pub fn validate_config<T: Into<PathBuf>>(config_path: T) -> DuckResult<()> {
    // Load and validate the configuration file.
    let loader = JsonConfigurationLoader::new(config_path.into());
    loader.load(&EnvironmentVariableProvider::new())?;
    Ok(())
}
