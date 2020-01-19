use std::sync::Arc;

use actix_web::{get, web};
use actix_web::{HttpResponse, Responder};

use crate::api::models::BuildViewModel;
use crate::engine::state::EngineState;

#[get("/builds")]
pub fn get_builds(state: web::Data<Arc<EngineState>>) -> impl Responder {
    // Convert to view models
    let builds: Vec<BuildViewModel> = state
        .builds
        .all()
        .iter()
        .map(BuildViewModel::from)
        .collect();

    // Serialize to JSON and return.
    let json = serde_json::to_string(&builds).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}
