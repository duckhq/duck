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
use crate::DuckResult;

pub async fn start_and_block(
    context: Arc<EngineState>,
    server_address: Option<String>,
) -> DuckResult<()> {
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
            .wrap(Cors::new().finish())
            .data(context.clone())
            .service(web::resource("/api/server").to(endpoints::server_info))
            .service(web::resource("/api/builds").to(endpoints::get_builds))
            .service(web::resource("/api/builds/view/{id}").to(endpoints::get_builds_for_view));

        if cfg!(feature = "docker") {
            // Serve static files from the ui directory.
            return app.service(fs::Files::new("/", "./web").index_file("index.html"));
        }

        return app;
    })
    .bind(bind)
    .unwrap()
    .run()
    .await?;

    Ok(())
}
