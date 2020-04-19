use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use actix_cors::Cors;
use actix_files as fs;
use actix_rt::System;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use actix_web_static_files;
use log::{debug, info};

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

///////////////////////////////////////////////////////////
// Handle

pub struct HttpServerHandle {
    server: actix_web::dev::Server,
}

impl HttpServerHandle {
    pub fn new(server: Server) -> Self {
        Self { server }
    }
    pub async fn stop(&self) {
        info!("Stopping HTTP server...");
        self.server.stop(true).await;
    }
}

///////////////////////////////////////////////////////////
// Start HTTP server

pub fn start(
    context: Arc<EngineState>,
    server_address: Option<String>,
) -> DuckResult<HttpServerHandle> {
    let bind = get_binding(&server_address);

    // Are we running embedded web?
    if cfg!(feature = "embedded-web") {
        debug!("Serving embedded UI.");
    }

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let system = System::new("duck-http-server");
        let server = HttpServer::new(move || {
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
        .bind(bind.clone())
        .unwrap()
        .disable_signals()
        .run();

        info!("HTTP server started: {}", bind);

        tx.send(server).unwrap();
        system.run()
    });

    Ok(HttpServerHandle::new(rx.recv()?))
}

fn get_binding(server_address: &Option<String>) -> String {
    // Get the address to bind to.
    match server_address {
        Some(ref address) => address.to_owned(),
        None => {
            if cfg!(feature = "docker") {
                // Bind to host container
                info!("Duck is compiled for docker, so binding to host container.");
                DOCKER_SERVER_ADDRESS.to_owned()
            } else if cfg!(feature = "embedded-web") {
                // Bind to port 8080
                info!("Duck is compiled with embedded UI, so binding to port 8080.");
                EMBEDDED_SERVER_ADDRESS.to_owned()
            } else {
                // Bind to localhost
                DEFAULT_SERVER_ADDRESS.to_owned()
            }
        }
    }
}
