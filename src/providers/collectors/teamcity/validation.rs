use url::Url;

use crate::config::{TeamCityAuth, TeamCityConfiguration, Validate};
use crate::utils::DuckResult;

impl Validate for TeamCityConfiguration {
    fn validate(&self) -> DuckResult<()> {
        self.credentials.validate()?;
        if self.id.is_empty() {
            return Err(format_err!("TeamCity collector have no ID."));
        }
        if let Err(e) = Url::parse(&self.server_url[..]) {
            return Err(format_err!("TeamCity server URL is invalid: {}", e));
        }
        Ok(())
    }
}

impl Validate for TeamCityAuth {
    fn validate(&self) -> DuckResult<()> {
        match self {
            TeamCityAuth::Guest => (),
            TeamCityAuth::BasicAuth { username, password } => {
                if username.is_empty() {
                    return Err(format_err!("TeamCity username cannot be empty."));
                }
                if password.is_empty() {
                    return Err(format_err!("TeamCity password cannot be empty."));
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
    fn should_return_error_if_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "",
                            "serverUrl": "https://localhost:5000",
                            "credentials": "guest",
                            "builds": [ "Foo" ]
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
    #[should_panic(expected = "TeamCity server URL is invalid: relative URL without a base")]
    fn should_return_error_if_teamcity_server_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "",
                            "credentials": "guest",
                            "builds": [ "Foo" ]
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
    #[should_panic(expected = "TeamCity username cannot be empty.")]
    fn should_return_error_if_teamcity_username_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "https://localhost:5000",
                            "credentials": {
                                "basic": {
                                    "username": "",
                                    "password": "bar"
                                }
                            },
                            "builds": [ "Foo" ]
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
    #[should_panic(expected = "TeamCity password cannot be empty.")]
    fn should_return_error_if_teamcity_password_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "https://localhost:5000",
                            "credentials": {
                                "basic": {
                                    "username": "john.doe",
                                    "password": ""
                                }
                            },
                            "builds": [ "Foo" ]
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
