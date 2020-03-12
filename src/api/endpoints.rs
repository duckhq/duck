use std::sync::Arc;

use actix_web::web;
use actix_web::HttpResponse;

use crate::engine::state::EngineState;
use crate::utils::VERSION;

use super::models::{BuildViewModel, ServerInfoModel, ViewInfoModel};

///////////////////////////////////////////////////////////
// Server information

pub async fn server_info(state: web::Data<Arc<EngineState>>) -> HttpResponse {
    let info = ServerInfoModel {
        title: &state.title[..],
        started: state
            .started
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: VERSION,
        views: state
            .views
            .get_views()
            .iter()
            .map(ViewInfoModel::from)
            .collect(),
    };
    let json = serde_json::to_string(&info).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}

///////////////////////////////////////////////////////////
// All builds

pub async fn get_builds(state: web::Data<Arc<EngineState>>) -> HttpResponse {
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

///////////////////////////////////////////////////////////
// Builds for view

pub async fn get_builds_for_view(
    id: web::Path<String>,
    state: web::Data<Arc<EngineState>>,
) -> HttpResponse {
    // Convert to view models
    let builds: Vec<BuildViewModel> = state
        .builds
        .for_view(&state.views, &id[..])
        .iter()
        .map(BuildViewModel::from)
        .collect();

    // Serialize to JSON and return.
    let json = serde_json::to_string(&builds).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json)
}
