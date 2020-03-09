use std::time::SystemTime;

use crate::config::Configuration;
use crate::engine::state::builds::BuildRepository;
use crate::engine::state::views::ViewRepository;

pub mod builds;
pub mod views;

pub struct EngineState {
    pub title: String,
    pub started: SystemTime,
    pub builds: BuildRepository,
    pub views: ViewRepository,
}

impl EngineState {
    pub fn new(config: &Configuration) -> Self {
        return EngineState {
            title: config.get_title().to_string(),
            started: SystemTime::now(),
            builds: BuildRepository::new(),
            views: ViewRepository::new(config),
        };
    }
}
