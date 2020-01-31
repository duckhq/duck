use url::Url;

use crate::config::{SlackConfiguration, SlackCredentials, Validate};
use crate::utils::DuckResult;

impl Validate for SlackConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.id.is_empty() {
            return Err(format_err!("Slack observer have no ID."));
        }
        match &self.credentials {
            SlackCredentials::Webhook { url } => {
                if let Err(e) = Url::parse(url) {
                    return Err(format_err!("Slack webhook URL is invalid: {}", e));
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Configuration;
    use crate::providers::DuckProviderCollection;
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid.")]
    fn should_return_error_if_slack_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "slack": {
                            "id": "",
                            "credentials": {
                                "webhook": {
                                    "url": "https://slack.com/MY-WEBHOOK-URL"
                                }
                            }
                        }
                    }
                ]
            }
        "#,
        )
        .unwrap();

        let collection = DuckProviderCollection::new();
        collection.get_observers(&config).unwrap();
    }

    #[test]
    #[should_panic(expected = "Slack webhook URL is invalid: relative URL without a base")]
    fn should_return_error_if_slack_webhook_url_is_invalid() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "slack": {
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

        let collection = DuckProviderCollection::new();
        collection.get_observers(&config).unwrap();
    }
}
