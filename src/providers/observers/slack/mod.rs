use std::collections::HashSet;
use std::iter::FromIterator;

use log::info;

use crate::builds::BuildStatus;
use crate::config::SlackConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;

use self::client::SlackClient;

mod client;
mod validation;

pub struct SlackObserver {
    client: SlackClient,
    info: ObserverInfo,
}

impl SlackObserver {
    pub fn new(config: &SlackConfiguration) -> Self {
        SlackObserver {
            client: SlackClient::new(config),
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

impl Observer for SlackObserver {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        if let Observation::BuildStatusChanged(build) = observation {
            if is_interesting_status(&build.status) {
                info!(
                    "Sending Slack message since build status changed ({:?})...",
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
                    match build.status {
                        BuildStatus::Success => ":heavy_check_mark:",
                        BuildStatus::Failed => ":heavy_multiplication_x:",
                        _ => ":question:",
                    },
                )?;
            }
        };

        Ok(())
    }
}

fn is_interesting_status(status: &BuildStatus) -> bool {
    match status {
        BuildStatus::Success | BuildStatus::Failed => true,
        _ => false,
    }
}
