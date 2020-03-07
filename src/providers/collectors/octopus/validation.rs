use url::Url;

use crate::config::{
    OctopusDeployConfiguration, OctopusDeployCredentials, OctopusDeployProject, Validate,
};
use crate::DuckResult;

impl Validate for OctopusDeployConfiguration {
    fn validate(&self) -> DuckResult<()> {
        self.credentials.validate()?;

        if self.id.is_empty() {
            return Err(format_err!("Octopus Deploy collector have no ID."));
        }
        if let Err(e) = Url::parse(&self.server_url[..]) {
            return Err(format_err!("Octopus Deploy server URL is invalid: {}", e));
        }

        if self.projects.is_empty() {
            return Err(format_err!(
                "Octopus Deploy collector '{}' have no configured projects.",
                self.id
            ));
        }
        for project in self.projects.iter() {
            project.validate()?;
        }

        Ok(())
    }
}

impl Validate for OctopusDeployCredentials {
    fn validate(&self) -> DuckResult<()> {
        match self {
            OctopusDeployCredentials::ApiKey(key) => {
                if key.is_empty() {
                    return Err(format_err!("Octopus Deploy API key is empty."));
                }
            }
        };
        Ok(())
    }
}

impl Validate for OctopusDeployProject {
    fn validate(&self) -> DuckResult<()> {
        if self.project_id.is_empty() {
            return Err(format_err!("Octopus Deploy project name is empty."));
        }
        if self.environments.is_empty() {
            return Err(format_err!(
                "The Octopus Deploy project '{}' contains no environments.",
                self.project_id
            ));
        }
        for environment in self.environments.iter() {
            if environment.is_empty() {
                return Err(format_err!(
                    "An Octopus Deploy environment in project '{}' is empty.",
                    self.project_id
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Configuration;
    use crate::providers;
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid.")]
    fn should_return_error_if_octopus_deploy_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "foo",
                                    "environments": [
                                        "Development", 
                                        "Staging",
                                        "Production"
                                    ]
                                }
                            ]
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
    #[should_panic(expected = "Octopus Deploy server URL is invalid: relative URL without a base")]
    fn should_return_error_if_server_url_is_empty_or_invalid() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "octopus",
                            "serverUrl": "",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "foo",
                                    "environments": [
                                        "Development", 
                                        "Staging",
                                        "Production"
                                    ]
                                }
                            ]
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
    #[should_panic(expected = "Octopus Deploy API key is empty.")]
    fn should_return_error_if_api_key_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "octopus",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": ""
                            },
                            "projects": [
                                {
                                    "projectId": "foo",
                                    "environments": [
                                        "Development", 
                                        "Staging",
                                        "Production"
                                    ]
                                }
                            ]
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
    #[should_panic(
        expected = "Octopus Deploy collector \\'octopus\\' have no configured projects."
    )]
    fn should_return_error_if_there_are_no_projects() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "octopus",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [ ]
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
    #[should_panic(expected = "Octopus Deploy project name is empty.")]
    fn should_return_error_if_project_name_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "octopus",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "",
                                    "environments": [
                                        "Development", 
                                        "Staging",
                                        "Production"
                                    ]
                                }
                            ]
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
    #[should_panic(expected = "An Octopus Deploy environment in project \\'foo\\' is empty.")]
    fn should_return_error_if_environment_name_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "octopus",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "foo",
                                    "environments": [
                                        "Development", 
                                        "Staging",
                                        ""
                                    ]
                                }
                            ]
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
    #[should_panic(expected = "The Octopus Deploy project \\'foo\\' contains no environments.")]
    fn should_return_error_if_there_are_no_environments_in_projects() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "octopus",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "foo",
                                    "environments": [ ]
                                }
                            ]
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
