use std::collections::HashMap;
use std::sync::Arc;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::web;
use actix_web::{App, HttpServer};
use actix_web_static_files;
use log::info;

mod endpoints;
mod models;

static DEFAULT_SERVER_ADDRESS: &str = "127.0.0.1:15825";
static EMBEDDED_SERVER_ADDRESS: &str = "127.0.0.1:8080";
static DOCKER_SERVER_ADDRESS: &str = "0.0.0.0:15825";

use crate::engine::state::EngineState;
use crate::DuckResult;

#[cfg(feature = "embedded-web")]
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

// Polyfill for building non embedded web.
#[cfg(not(feature = "embedded-web"))]
use actix_web_static_files::Resource;
#[cfg(not(feature = "embedded-web"))]
fn generate() -> HashMap<&'static str, Resource> {
    HashMap::new()
}

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
            } else if cfg!(feature = "embedded-web") {
                // Bind to port 8080
                info!("Duck is compiled with embedded UI, so binding to port 8080.");
                EMBEDDED_SERVER_ADDRESS
            } else {
                // Bind to localhost
                DEFAULT_SERVER_ADDRESS
            }
        }
    };

    // Are we running embedded web?
    if cfg!(feature = "embedded-web") {
        info!("Serving embedded UI.");
    }

    info!("Duck server address: {}", bind);

    HttpServer::new(move || {
        let app = App::new()
            .wrap(Cors::new().finish())
            .data(context.clone())
            .service(web::resource("/api/server").to(endpoints::server_info))
            .service(web::resource("/api/builds").to(endpoints::get_builds))
            .service(web::resource("/api/builds/view/{id}").to(endpoints::get_builds_for_view));

        // Serve static files from the web directory?
        if cfg!(feature = "docker") {
            return app.service(fs::Files::new("/", "./web").index_file("index.html"));
        }

        // Serve embedded web?
        if cfg!(feature = "embedded-web") {
            let generated = generate();
            return app.service(actix_web_static_files::ResourceFiles::new("/", generated));
        }

        return app;
    })
    .bind(bind)
    .unwrap()
    .run()
    .await?;

    Ok(())
}
