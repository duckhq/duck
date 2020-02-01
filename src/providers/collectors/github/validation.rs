use crate::config::{GitHubConfiguration, GitHubCredentials, Validate};
use crate::utils::DuckResult;

impl Validate for GitHubConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.owner.is_empty() {
            return Err(format_err!("GitHub owner is empty."));
        }
        if self.repository.is_empty() {
            return Err(format_err!("GitHub repository is empty."));
        }
        if self.workflow.is_empty() {
            return Err(format_err!("GitHub workflow is empty."));
        }

        match &self.credentials {
            GitHubCredentials::Basic { username, password } => {
                if username.is_empty() {
                    return Err(format_err!("GitHub username is empty."));
                }
                if password.is_empty() {
                    return Err(format_err!("GitHub password is empty."));
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
    fn should_return_error_if_github_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "github": {
                            "id": "",
                            "owner": "spectresystems",
                            "repository": "duck",
                            "workflow": "pull_request.yml",
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
    #[should_panic(expected = "GitHub owner is empty.")]
    fn should_return_error_if_github_owner_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "github": {
                            "id": "duck_pullrequests",
                            "owner": "",
                            "repository": "duck",
                            "workflow": "pull_request.yml",
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
    #[should_panic(expected = "GitHub repository is empty.")]
    fn should_return_error_if_github_repository_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "github": {
                            "id": "duck_pullrequests",
                            "owner": "spectresystems",
                            "repository": "",
                            "workflow": "pull_request.yml",
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
    #[should_panic(expected = "GitHub workflow is empty.")]
    fn should_return_error_if_github_workflow_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "github": {
                            "id": "duck_pullrequests",
                            "owner": "spectresystems",
                            "repository": "duck",
                            "workflow": "",
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
    #[should_panic(expected = "GitHub username is empty.")]
    fn should_return_error_if_github_username_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "github": {
                            "id": "duck_pullrequests",
                            "owner": "spectresystems",
                            "repository": "duck",
                            "workflow": "pull_request.yml",
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
    #[should_panic(expected = "GitHub password is empty.")]
    fn should_return_error_if_github_password_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "github": {
                            "id": "duck_pullrequests",
                            "owner": "spectresystems",
                            "repository": "duck",
                            "workflow": "pull_request.yml",
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
