use actix_cors::Cors;
use actix_files as fs;
use actix_web::web;
use actix_web::{App, HttpServer};
use log::info;
use std::sync::Arc;

mod endpoints;
mod models;

static DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:15825";
static DOCKER_SERVER_ADDRESS: &str = "0.0.0.0:15825";

use crate::engine::state::EngineState;
use crate::utils::DuckResult;

pub fn start_and_block(
    context: Arc<EngineState>,
    server_address: Option<String>,
) -> DuckResult<()> {
    let state = web::Data::new(context);

    // Get the address to bind to.
    let bind = match server_address {
        Some(ref address) => address,
        None => {
            if cfg!(feature = "docker") {
                // Bind to host container
                info!("Duck is compiled for docker, so binding to host container.");
                DOCKER_SERVER_ADDRESS
            } else {
                // Bind to localhost
                DEFAULT_SERVER_ADDRESS
            }
        }
    };

    info!("Duck server address: {}", bind);

    HttpServer::new(move || {
        let app = App::new()
            .register_data(state.clone())
            .wrap(Cors::new())
            .service(endpoints::server_info)
            .service(endpoints::get_builds)
            .service(endpoints::get_builds_for_view);

        if cfg!(feature = "docker") {
            // Serve static files from the ui directory.
            return app.service(fs::Files::new("/", "./web").index_file("index.html"));
        }

        return app;
    })
    .bind(bind)
    .unwrap()
    .run()?;

    Ok(())
}
