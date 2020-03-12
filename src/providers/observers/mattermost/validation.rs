use url::Url;

use crate::config::{MattermostConfiguration, MattermostCredentials, Validate};
use crate::DuckResult;

impl Validate for MattermostConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if let Some(channel) = &self.channel {
            if channel.is_empty() {
                return Err(format_err!("[{}] Mattermost channel is empty", self.id));
            }
        }

        match &self.credentials {
            MattermostCredentials::Webhook { url } => {
                if let Err(e) = Url::parse(url) {
                    return Err(format_err!(
                        "[{}] Mattermost webhook URL is invalid: {}",
                        self.id,
                        e
                    ));
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Configuration;
    use crate::providers;
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid")]
    fn should_return_error_if_mattermost_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
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

        providers::create_observers(&config).unwrap();
    }

    #[test]
    #[should_panic(expected = "[foo] Mattermost channel is empty")]
    fn should_return_error_if_mattermost_channel_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "mattermost": {
                            "id": "foo",
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

        providers::create_observers(&config).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "[foo] Mattermost webhook URL is invalid: relative URL without a base"
    )]
    fn should_return_error_if_mattermost_webhook_url_is_invalid() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "mattermost": {
                            "id": "foo",
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

        providers::create_observers(&config).unwrap();
    }
}
