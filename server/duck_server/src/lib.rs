#[macro_use]
extern crate serde;
extern crate log;

pub mod config;
mod engine;
mod utils;
mod web;

use failure::Error;

use config::ConfigurationLoader;

pub type DuckResult<T> = Result<T, Error>;

pub async fn run(config: impl ConfigurationLoader) -> DuckResult<()> {
    println!(r#"     ____             __  "#);
    println!(r#"    / __ \__  _______/ /__"#);
    println!(r#"   / / / / / / / ___/ //_/"#);
    println!(r#"  / /_/ / /_/ / /__/ ,<   "#);
    println!(r#" /_____/\__,_/\___/_/|_|  "#);
    println!();

    // Start engine.
    let engine = engine::Engine::new(&config)?;
    let engine_handle = engine.run()?;

    // Start web server
    web::start().await?;

    // Stop the engine.
    engine_handle.stop()?;

    Ok(())
}
