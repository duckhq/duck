use crate::config::{JenkinsConfiguration, JenkinsCredentials, Validate};
use crate::utils::DuckResult;

impl Validate for JenkinsConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.server_url.is_empty() {
            return Err(format_err!("Server URL is empty."));
        }

        match &self.credentials {
            JenkinsCredentials::Basic { username, password } => {
                if username.is_empty() {
                    return Err(format_err!("Jenkins username is empty."));
                }
                if password.is_empty() {
                    return Err(format_err!("Jenkins password is empty."));
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::*;
    use crate::providers::DuckProviderCollection;
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid.")]
    fn should_return_error_if_jenkins_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "jenkins": {
                            "id": "",
                            "serverUrl": "http://jenkins:8080",
                            "credentials": {
                                "basic": {
                                    "username": "patrik",
                                    "password": "hunter1!"
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
        collection.get_collectors(&config).unwrap();
    }
    #[test]
    #[should_panic(expected = "Jenkins server url is empty.")]
    fn should_return_error_if_jenkins_repository_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "jenkins": {
                            "id": "duck_pipelines",
                            "serverUrl": "",
                            "credentials": {
                                "basic": {
                                    "username": "patrik",
                                    "password": "hunter1!"
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
        collection.get_collectors(&config).unwrap();
    }

    #[test]
    #[should_panic(expected = "Jenkins username is empty.")]
    fn should_return_error_if_jenkins_username_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "jenkins": {
                            "id": "duck_pipelines",
                            "serverUrl": "http://jenkins:8080",
                            "credentials": 
                            {
                                "basic": {
                                    "username": "",
                                    "password": "hunter1!"
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
        collection.get_collectors(&config).unwrap();
    }

    #[test]
    #[should_panic(expected = "Jenkins password is empty.")]
    fn should_return_error_if_jenkins_password_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "jenkins": {
                            "id": "duck_pipelines",
                            "serverUrl": "http://jenkins:8080",
                            "credentials": 
                            {
                                "basic": {
                                    "username": "patriksvensson",
                                    "password": ""
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
        collection.get_collectors(&config).unwrap();
    }
}
