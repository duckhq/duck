use url::Url;

use crate::config::{OctopusDeployConfiguration, OctopusDeployCredentials, Validate};
use crate::DuckResult;

impl Validate for OctopusDeployConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if let Err(e) = Url::parse(&self.server_url[..]) {
            return Err(format_err!(
                "[{}] Octopus Deploy server URL is invalid: {}",
                self.id,
                e
            ));
        }

        if self.projects.is_empty() {
            return Err(format_err!(
                "[{}] Octopus Deploy projects are empty",
                self.id
            ));
        }

        for project in self.projects.iter() {
            if project.project_id.is_empty() {
                return Err(format_err!(
                    "[{}] Octopus Deploy project name is empty",
                    self.id
                ));
            }
            if project.environments.is_empty() {
                return Err(format_err!(
                    "[{}] The Octopus Deploy project '{}' contains no environments",
                    self.id,
                    project.project_id
                ));
            }
            for environment in project.environments.iter() {
                if environment.is_empty() {
                    return Err(format_err!(
                        "[{}] An Octopus Deploy environment in project '{}' is empty",
                        self.id,
                        project.project_id
                    ));
                }
            }
        }

        match &self.credentials {
            OctopusDeployCredentials::ApiKey(key) => {
                if key.is_empty() {
                    return Err(format_err!("[{}] Octopus Deploy API key is empty", self.id));
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
    #[should_panic(
        expected = "[foo] Octopus Deploy server URL is invalid: relative URL without a base"
    )]
    fn should_return_error_if_server_url_is_empty_or_invalid() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "foo",
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
    #[should_panic(expected = "[foo] Octopus Deploy API key is empty")]
    fn should_return_error_if_api_key_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "foo",
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
    #[should_panic(expected = "[foo] Octopus Deploy projects are empty")]
    fn should_return_error_if_there_are_no_projects() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "foo",
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
    #[should_panic(expected = "[foo] Octopus Deploy project name is empty")]
    fn should_return_error_if_project_name_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "foo",
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
    #[should_panic(expected = "[foo] An Octopus Deploy environment in project \\'bar\\' is empty")]
    fn should_return_error_if_environment_name_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "foo",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "bar",
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
    #[should_panic(
        expected = "[foo] The Octopus Deploy project \\'bar\\' contains no environments"
    )]
    fn should_return_error_if_there_are_no_environments_in_projects() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "octopus": {
                            "id": "foo",
                            "serverUrl": "http://localhost:9000",
                            "credentials": {
                                "apiKey": "MY-SECRET-API-KEY"
                            },
                            "projects": [
                                {
                                    "projectId": "bar",
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
