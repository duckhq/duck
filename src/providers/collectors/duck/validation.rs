use crate::config::{DuckConfiguration, Validate};
use crate::DuckResult;

impl Validate for DuckConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.server_url.is_empty() {
            return Err(format_err!("[{}] Duck server URL is empty", self.id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::*;
    use crate::providers;
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid")]
    fn should_return_error_if_github_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "duck": {
                            "id": "",
                            "serverUrl": "http://127.0.0.1:8081"
                        }
                    }
                ] 
            }
        "#,
        )
        .unwrap();

        providers::create_collectors(&config).unwrap();
    }

    #[test]
    #[should_panic(expected = "[duck_other] Duck server URL is empty")]
    fn should_return_error_if_github_owner_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "duck": {
                            "id": "duck_other",
                            "serverUrl": ""
                        }
                    }
                ] 
            }
        "#,
        )
        .unwrap();

        providers::create_collectors(&config).unwrap();
    }
}
