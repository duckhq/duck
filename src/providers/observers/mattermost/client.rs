use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, ClientBuilder};

use crate::config::{MattermostConfiguration, MattermostCredentials};
use crate::utils::DuckResult;

pub struct MattermostClient {
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
