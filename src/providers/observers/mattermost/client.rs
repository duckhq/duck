use crate::config::{MattermostConfiguration, MattermostCredentials};
use crate::utils::http::{HttpClient, HttpRequestBuilder, HttpResponse};
use crate::DuckResult;

pub struct MattermostClient {
    channel: Option<String>,
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
            credentials: config.credentials.clone(),
        }
    }

    pub fn send(&self, client: &impl HttpClient, message: &str) -> DuckResult<()> {
        let mut builder = HttpRequestBuilder::post(self.credentials.get_url().to_string());
        builder.add_header("Content-Type", "application/json");
        builder.add_header("Accept", "application/json");
        builder.set_body(self.get_payload(message).to_string());

        let response = client.send(&builder)?;
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
