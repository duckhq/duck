use crate::config::{DebuggerConfiguration, Validate};
use crate::DuckResult;

impl Validate for DebuggerConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.server_url.is_empty() {
            return Err(format_err!("[{}] Debugger server URL is empty", self.id));
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
    fn should_return_error_if_duck_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "debugger": {
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
    #[should_panic(expected = "[duck_debugger] Debugger server URL is empty")]
    fn should_return_error_if_duck_server_url_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "debugger": {
                            "id": "duck_debugger",
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
