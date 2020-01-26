use crate::config::{SlackConfiguration, SlackCredentials};
use crate::utils::http::{HttpClient, HttpRequestBuilder, HttpResponse};
use crate::utils::DuckResult;

pub struct SlackClient {
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
            credentials: config.credentials.clone(),
        }
    }

    pub fn send(&self, client: &impl HttpClient, message: &str, icon: &str) -> DuckResult<()> {
        let mut builder = HttpRequestBuilder::put(self.credentials.get_url().to_string());
        builder.add_header("Content-Type", "application/json");
        builder.add_header("Accept", "application/json");
        builder.set_body(
            json!({
                "username": "Duck",
                "icon_emoji": icon,
                "text": message
            })
            .to_string(),
        );

        let response = client.send(&builder)?;
        if !response.status().is_success() {
            return Err(format_err!(
                "Could not send Slack message ({})",
                response.status()
            ));
        }

        Ok(())
    }
}
