use std::collections::HashSet;
use std::iter::FromIterator;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, ClientBuilder};

use log::{error, info};

use crate::builds::{Build, BuildStatus};
use crate::config::{SlackConfiguration, SlackCredentials};
use crate::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;

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
            if build.status != BuildStatus::Unknown {
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
                    get_message_icon(&build),
                )?;
            }
        };

        Ok(())
    }
}

fn get_message_icon(build: &Build) -> &str {
    match build.status {
        BuildStatus::Success => ":heavy_check_mark:",
        BuildStatus::Failed => ":heavy_multiplication_x:",
        BuildStatus::Running => ":shipit:",
        BuildStatus::Unknown => ":question:",
    }
}

struct SlackClient {
    client: Client,
    credentials: SlackCredentials,
}

impl SlackCredentials {
    pub fn get_url(&self) -> &str {
        match self {
            SlackCredentials::Webhook { url } => &url[..],
        }
    }
}

impl SlackClient {
    pub fn new(config: &SlackConfiguration) -> Self {
        SlackClient {
            client: ClientBuilder::new().build().unwrap(),
            credentials: config.credentials.clone(),
        }
    }

    pub fn send(&self, message: &str, icon: &str) -> DuckResult<()> {
        let response = self
            .client
            .put(self.credentials.get_url())
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(
                json!({
                    "username": "Duck",
                    "icon_emoji": icon,
                    "text": message
                })
                .to_string(),
            )
            .send()?;

        if !response.status().is_success() {
            error!("Could not send Slack message ({})!", response.status());
        }

        Ok(())
    }
}
