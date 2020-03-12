use crate::config::{AppVeyorConfiguration, Validate};
use crate::DuckResult;

impl Validate for AppVeyorConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.account.is_empty() {
            return Err(format_err!("[{}] AppVeyor account is empty", self.id));
        }
        if self.project.is_empty() {
            return Err(format_err!("[{}] AppVeyor project is empty", self.id));
        }
        match &self.credentials {
            crate::config::AppVeyorCredentials::Bearer(token) => {
                if token.is_empty() {
                    return Err(format_err!("[{}] AppVeyor bearer token is empty", self.id));
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::*;
    use crate::providers;
    use crate::providers::collectors::Collector;
    use crate::utils::text::TestVariableProvider;

    fn create_collectors_from_config(json: &str) -> Vec<Box<dyn Collector>> {
        providers::create_collectors(
            &Configuration::from_json(&TestVariableProvider::new(), json).unwrap(),
        )
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid")]
    fn should_return_error_if_id_is_empty() {
        create_collectors_from_config(
            r#"
        { 
            "collectors": [ 
                {
                    "appveyor": {
                        "id": "",
                        "credentials": {
                            "bearer": "SECRET"
                        },
                        "account": "patriksvensson",
                        "project": "spectre-commandline",
                        "count": 5
                    }
                }
            ] 
        }"#,
        );
    }

    #[test]
    #[should_panic(expected = "[appveyor_spectrecli] AppVeyor bearer token is empty")]
    fn should_return_error_if_credentials_is_empty() {
        create_collectors_from_config(
            r#"
        { 
            "collectors": [ 
                {
                    "appveyor": {
                        "id": "appveyor_spectrecli",
                        "credentials": {
                            "bearer": ""
                        },
                        "account": "patriksvensson",
                        "project": "spectre-commandline",
                        "count": 5
                    }
                }
            ] 
        }"#,
        );
    }

    #[test]
    #[should_panic(expected = "[appveyor_spectrecli] AppVeyor account is empty")]
    fn should_return_error_if_account_is_empty() {
        create_collectors_from_config(
            r#"
        { 
            "collectors": [ 
                {
                    "appveyor": {
                        "id": "appveyor_spectrecli",
                        "credentials": {
                            "bearer": ""
                        },
                        "account": "",
                        "project": "spectre-commandline",
                        "count": 5
                    }
                }
            ] 
        }"#,
        );
    }

    #[test]
    #[should_panic(expected = "[appveyor_spectrecli] AppVeyor project is empty")]
    fn should_return_error_if_project_is_empty() {
        create_collectors_from_config(
            r#"
        { 
            "collectors": [ 
                {
                    "appveyor": {
                        "id": "appveyor_spectrecli",
                        "credentials": {
                            "bearer": ""
                        },
                        "account": "patriksvensson",
                        "project": "",
                        "count": 5
                    }
                }
            ] 
        }"#,
        );
    }
}
