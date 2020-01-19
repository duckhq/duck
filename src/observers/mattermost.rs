use std::collections::HashSet;
use std::iter::FromIterator;

use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, ClientBuilder};

use log::info;

use crate::builds::BuildStatus;
use crate::config::{MattermostConfiguration, MattermostCredentials};
use crate::observers::{Observation, Observer, ObserverInfo};
use crate::utils::DuckResult;

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
        if let Observation::BuildUpdated(build) = observation {
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

struct MattermostClient {
    channel: Option<String>,
    client: Client,
    credentials: MattermostCredentials,
}

impl MattermostCredentials {
    fn get_url(&self) -> &str {
        match self {
            MattermostCredentials::Webhook { url } => url,
        }
    }
}

impl MattermostClient {
    pub fn new(config: &MattermostConfiguration) -> Self {
        MattermostClient {
            channel: config.channel.clone(),
            client: ClientBuilder::new().build().unwrap(),
            credentials: config.credentials.clone(),
        }
    }

    pub fn send(&self, message: &str) -> DuckResult<()> {
        let request = self
            .client
            .post(self.credentials.get_url())
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .body(self.get_payload(message).to_string());

        let response = request.send()?;
        if !response.status().is_success() {
            return Err(format_err!(
                "Could not send Mattermost message. ({})",
                response.status()
            ));
        }

        Ok(())
    }

    fn get_payload(&self, message: &str) -> serde_json::Value {
        match self.channel {
            Option::None => json!({ "text": message }),
            Option::Some(_) => json!({
                "channel_id": self.channel,
                "text": message
            }),
        }
    }
}
