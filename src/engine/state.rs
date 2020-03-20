use std::time::SystemTime;

use log::debug;

use crate::config::Configuration;
use crate::engine::state::builds::BuildRepository;
use crate::engine::state::ui::UiRepository;
use crate::engine::state::views::ViewRepository;

pub mod builds;
pub mod ui;
pub mod views;

pub struct EngineState {
    pub started: SystemTime,
    pub builds: BuildRepository,
    pub ui: UiRepository,
    pub views: ViewRepository,
}

impl EngineState {
    pub fn new() -> Self {
        return EngineState {
            started: SystemTime::now(),
            builds: BuildRepository::new(),
            ui: UiRepository::new(),
            views: ViewRepository::new(),
        };
    }

    pub fn refresh(&self, config: &Configuration) {
        debug!("Refreshing configuration");
        self.ui.set_title(&config.title[..]);
        if let Some(views) = &config.views {
            self.views.add_views(views);
        }
    }
}
