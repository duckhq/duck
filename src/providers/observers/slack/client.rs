use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, ClientBuilder};

use log::error;

use crate::config::{SlackConfiguration, SlackCredentials};
use crate::utils::DuckResult;

pub struct SlackClient {
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
