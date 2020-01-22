use std::collections::HashSet;
use std::iter::FromIterator;

use log::info;

use crate::config::HueConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;

use self::client::HueClient;

mod client;
mod validation;

pub struct HueObserver {
    info: ObserverInfo,
    client: HueClient,
}

impl HueObserver {
    pub fn new(config: &HueConfiguration) -> Self {
        HueObserver {
            client: HueClient::new(config),
            info: ObserverInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    None => true,
                    Some(e) => e,
                },
                collectors: match &config.collectors {
                    Option::None => Option::None,
                    Option::Some(collectors) => {
                        Some(HashSet::from_iter(collectors.iter().cloned()))
                    }
                },
            },
        }
    }
}

impl Observer for HueObserver {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        match observation {
            Observation::DuckStatusChanged(status) => {
                info!(
                    "[{}] Setting light state to '{:?}'...",
                    self.info.id, status
                );
                self.client.set_state(status)?;
            }
            Observation::ShuttingDown => {
                info!("[{}] Turning off all lights...", self.info.id);
                self.client.turn_off()?;
            }
            _ => {}
        }
        Ok(())
    }
}
