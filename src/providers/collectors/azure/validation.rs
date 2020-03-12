use crate::config::{AzureDevOpsConfiguration, AzureDevOpsCredentials, Validate};
use crate::DuckResult;

impl Validate for AzureDevOpsConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.organization.is_empty() {
            return Err(format_err!(
                "[{}] Azure DevOps organization is empty",
                self.id
            ));
        }
        if self.project.is_empty() {
            return Err(format_err!("[{}] Azure DevOps project is empty", self.id));
        }
        if self.definitions.is_empty() {
            return Err(format_err!(
                "[{}] Azure DevOps configuration have not specified any build definitions",
                self.id
            ));
        }
        if self.branches.is_empty() {
            return Err(format_err!(
                "[{}] Azure DevOps configuration have not specified any branches",
                self.id
            ));
        }

        match &self.credentials {
            AzureDevOpsCredentials::Anonymous => {}
            AzureDevOpsCredentials::PersonalAccessToken(token) => {
                if token.is_empty() {
                    return Err(format_err!("[{}] Azure DevOps PAT token is empty", self.id));
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
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "The id \\'\\' is invalid")]
    fn should_return_error_if_azure_devops_id_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "azure": {
                            "id": "",
                            "organization": "cake-build",
                            "project": "Cake",
                            "credentials": "anonymous",
                            "definitions": [ "1", "3", "5" ],
                            "branches": [ "refs/heads/develop" ]
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
    #[should_panic(expected = "[foo] Azure DevOps organization is empty")]
    fn should_return_error_if_azure_devops_organization_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "azure": {
                            "id": "foo",
                            "organization": "",
                            "project": "Cake",
                            "credentials": "anonymous",
                            "definitions": [ "1", "3", "5" ],
                            "branches": [ "refs/heads/develop" ]
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
    #[should_panic(expected = "[foo] Azure DevOps project is empty")]
    fn should_return_error_if_azure_devops_project_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "azure": {
                            "id": "foo",
                            "organization": "cake-build",
                            "project": "",
                            "credentials": "anonymous",
                            "definitions": [ "1", "3", "5" ],
                            "branches": [ "refs/heads/develop" ]
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
        expected = "[foo] Azure DevOps configuration have not specified any build definitions"
    )]
    fn should_return_error_if_azure_devops_definitions_are_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "azure": {
                            "id": "foo",
                            "organization": "cake-build",
                            "project": "Cake",
                            "credentials": "anonymous",
                            "definitions": [ ],
                            "branches": [ "refs/heads/develop" ]
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
    #[should_panic(expected = "[foo] Azure DevOps configuration have not specified any branches")]
    fn should_return_error_if_azure_devops_branches_are_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "azure": {
                            "id": "foo",
                            "organization": "cake-build",
                            "project": "Cake",
                            "credentials": "anonymous",
                            "definitions": [ "1", "3", "5" ],
                            "branches": [ ]
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
    #[should_panic(expected = "[foo] Azure DevOps PAT token is empty")]
    fn should_return_error_if_azure_devops_token_is_empty() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "azure": {
                            "id": "foo",
                            "organization": "cake-build",
                            "project": "Cake",
                            "credentials": {
                                "pat": ""
                            },
                            "definitions": [ "1", "3", "5" ],
                            "branches": [ "refs/heads/develop" ]
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
