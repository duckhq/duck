use std::collections::HashSet;
use std::iter::FromIterator;

use log::info;

use crate::builds::BuildStatus;
use crate::config::MattermostConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;

use self::client::MattermostClient;

mod client;
mod validation;

pub struct MattermostObserver {
    client: MattermostClient,
    info: ObserverInfo,
}

impl MattermostObserver {
    pub fn new(config: &MattermostConfiguration) -> Self {
        MattermostObserver {
            client: MattermostClient::new(config),
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

impl Observer for MattermostObserver {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        if let Observation::BuildStatusChanged(build) = observation {
            if build.status != BuildStatus::Unknown {
                info!(
                    "Sending Mattermost message since build status changed ({:?})...",
                    build.status
                );
                self.client.send(
                    &format!(
                        "{:?} build status for {}::{} ({}) changed to *{:?}*",
                        build.provider,
                        build.project_name,
                        build.definition_name,
                        build.branch,
                        build.status
                    )[..],
                )?;
            }
        };

        Ok(())
    }
}
