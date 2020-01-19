use url::Url;

use crate::config::{MattermostConfiguration, MattermostCredentials, Validate};
use crate::utils::DuckResult;

impl Validate for MattermostConfiguration {
    fn validate(&self) -> DuckResult<()> {
        self.credentials.validate()?;
        if self.id.is_empty() {
            return Err(format_err!("Mattermost observer have no ID."));
        }
        if let Some(channel) = &self.channel {
            if channel.is_empty() {
                return Err(format_err!("Mattermost channel is empty."));
            }
        }
        Ok(())
    }
}

impl Validate for MattermostCredentials {
    fn validate(&self) -> DuckResult<()> {
        match self {
            MattermostCredentials::Webhook { url } => {
                if let Err(e) = Url::parse(url) {
                    return Err(format_err!("Mattermost webhook URL is invalid: {}", e));
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Configuration;

    #[test]
    #[should_panic(expected = "Mattermost observer have no ID.")]
    fn should_return_error_if_mattermost_id_is_empty() {
        let config = Configuration::from_json(
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "mattermost": {
                            "id": "",
                            "credentials": {
                                "webhook": {
                                    "url": "https://mattermost.example.com"
                                }
                            }
                        }             
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "Mattermost channel is empty.")]
    fn should_return_error_if_mattermost_channel_is_empty() {
        let config = Configuration::from_json(
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "mattermost": {
                            "id": "bar",
                            "channel": "",
                            "credentials": {
                                "webhook": {
                                    "url": "https://mattermost.example.com"
                                }
                            }
                        }             
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "Mattermost webhook URL is invalid: relative URL without a base")]
    fn should_return_error_if_mattermost_webhook_url_is_invalid() {
        let config = Configuration::from_json(
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "mattermost": {
                            "id": "bar",
                            "credentials": {
                                "webhook": {
                                    "url": ""
                                }
                            }
                        }             
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }
}
