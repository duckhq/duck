use std::sync::Arc;

use actix_web::{web, App, HttpResponse, HttpServer};

use crate::state::State;
use crate::DuckResult;

pub async fn start(state: Arc<State>) -> DuckResult<()> {
    Ok(HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(web::resource("/api/server").to(server_info))
    })
    .bind("127.0.0.1:15825")?
    .run()
    .await?)
}

///////////////////////////////////////////////////////////
// View models

#[derive(Serialize, Clone)]
struct ServerInfoViewModel<'a> {
    pub title: &'a str,
}

///////////////////////////////////////////////////////////
// Endpoints

async fn server_info(state: web::Data<Arc<State>>) -> HttpResponse {
    let title = state.title.lock().unwrap();
    let view_model = ServerInfoViewModel { title: &title };
    let json = serde_json::to_string(&view_model).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}
