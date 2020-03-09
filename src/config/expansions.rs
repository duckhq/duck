///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use crate::config::*;
    use crate::utils::text::*;

    static CONFIGURATION: &str = r#"
    { 
        "collectors": [ 
            {
                "teamcity": {
                    "id": "${TEAMCITY_ID}",
                    "serverUrl": "https://${TEAMCITY_HOST}:${TEAMCITY_PORT}",
                    "credentials": {
                        "basic": {
                            "username": "${TEAMCITY_USERNAME}",
                            "password": "${TEAMCITY_PASSWORD}"
                        }
                    },
                    "builds": [ "${TEAMCITY_BUILD}_1", "${TEAMCITY_BUILD}_2" ]
                }
            },
            {
                "azure": {
                    "id": "${AZURE_ID}",
                    "organization": "${AZURE_ORG}",
                    "project": "${AZURE_PROJECT}",
                    "credentials": {
                        "pat": "${AZURE_PAT}"
                    },
                    "branches": [ "${AZURE_BRANCH}_1", "${AZURE_BRANCH}_2" ],
                    "definitions": [ "${AZURE_DEF}_1", "${AZURE_DEF}_2" ]
                }
            },
            {
                "github": {
                    "id": "${GITHUB_ID}",
                    "owner": "${GITHUB_OWNER}",
                    "repository": "${GITHUB_REPOSITORY}",
                    "workflow": "${GITHUB_WORKFLOW}",
                    "credentials": {
                        "basic": {
                            "username": "${GITHUB_USERNAME}",
                            "password": "${GITHUB_PASSWORD}"
                        }
                    }
                }
            },
            {
                "octopus": {
                    "id": "${OCTOPUS_ID}",
                    "serverUrl": "https://${OCTOPUS_HOST}:${OCTOPUS_PORT}",
                    "credentials": {
                        "apiKey": "${OCTOPUS_API_KEY}"
                    },
                    "projects": [
                        {
                            "projectId": "${OCTOPUS_PROJECT_PREFIX}-1",
                            "environments": [
                                "${OCTOPUS_ENVIRONMENT_PREFIX}-1", 
                                "${OCTOPUS_ENVIRONMENT_PREFIX}-2"
                            ]
                        }
                    ]
                }
            }
        ],
        "observers": [
            {
                "hue": {
                    "id": "${HUE_ID}",
                    "hubUrl": "https://${HUE_HOST}",
                    "username": "${HUE_USERNAME}",
                    "brightness": ${HUE_BRIGHTNESS},
                    "lights": [ 
                        "${HUE_LIGHT_PREFIX}_1", 
                        "${HUE_LIGHT_PREFIX}_2" 
                    ]
                }
            },
            {
                "slack": {
                    "id": "${SLACK_ID}",
                    "credentials": {
                        "webhook": {
                            "url": "${SLACK_WEBHOOK_URL}"
                        }
                    }
                }
            },
            {
                "mattermost": {
                    "id": "${MATTERMOST_ID}",
                    "channel": "${MATTERMOST_CHANNEL}",
                    "credentials": {
                        "webhook": {
                            "url": "${MATTERMOST_WEBHOOK_URL}"
                        }
                    }
                }
            }
        ]
    }
    "#;

    macro_rules! find_config {
        ($t:expr, $q:path) => {
            $t.iter()
                .find_map(|i| match i {
                    $q(c) => Some(c),
                    _ => None,
                })
                .expect("Could not find configuration.");
        };
    }

    macro_rules! read_config {
        ($t:expr) => {
            Configuration::from_json(&create_variables(), $t).unwrap();
        };
    }

    fn create_variables() -> TestVariableProvider {
        let mut variables = TestVariableProvider::new();
        variables.add("TEAMCITY_ID", "teamcity");
        variables.add("TEAMCITY_HOST", "localhost");
        variables.add("TEAMCITY_PORT", "8111");
        variables.add("TEAMCITY_BUILD", "MYBUILD");
        variables.add("TEAMCITY_USERNAME", "patrik");
        variables.add("TEAMCITY_PASSWORD", "hunter1!");
        variables.add("AZURE_ID", "azure");
        variables.add("AZURE_ORG", "MyOrganization");
        variables.add("AZURE_PROJECT", "MyProject");
        variables.add("AZURE_PAT", "SECRET-PAT-TOKEN");
        variables.add("AZURE_BRANCH", "MyBranch");
        variables.add("AZURE_DEF", "MyDefinition");
        variables.add("GITHUB_ID", "github");
        variables.add("GITHUB_OWNER", "spectresystems");
        variables.add("GITHUB_REPOSITORY", "duck");
        variables.add("GITHUB_WORKFLOW", "workflow.yml");
        variables.add("GITHUB_USERNAME", "patrik");
        variables.add("GITHUB_PASSWORD", "hunter1!");
        variables.add("OCTOPUS_ID", "octopus");
        variables.add("OCTOPUS_HOST", "localhost");
        variables.add("OCTOPUS_PORT", "9000");
        variables.add("OCTOPUS_PROJECT_PREFIX", "Projects");
        variables.add("OCTOPUS_ENVIRONMENT_PREFIX", "Environments");
        variables.add("OCTOPUS_API_KEY", "SECRET-API-KEY");
        variables.add("HUE_ID", "hue");
        variables.add("HUE_BRIGHTNESS", "128");
        variables.add("HUE_HOST", "192.168.1.155");
        variables.add("HUE_USERNAME", "patrik");
        variables.add("HUE_LIGHT_PREFIX", "Light");
        variables.add("SLACK_ID", "slack");
        variables.add("SLACK_WEBHOOK_URL", "https://example.com/Slack");
        variables.add("MATTERMOST_ID", "mattermost");
        variables.add("MATTERMOST_CHANNEL", "some-channel");
        variables.add("MATTERMOST_WEBHOOK_URL", "https://example.com/mattermost");
        return variables;
    }

    #[test]
    fn should_expand_teamcity_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let teamcity = find_config!(config.collectors, CollectorConfiguration::TeamCity);
        let (username, password) = teamcity.get_basic_auth();

        assert_eq!("teamcity", teamcity.id);
        assert_eq!("https://localhost:8111", teamcity.server_url);
        assert_eq!("MYBUILD_1", teamcity.builds[0]);
        assert_eq!("MYBUILD_2", teamcity.builds[1]);
        assert_eq!("patrik", username);
        assert_eq!("hunter1!", password);
    }

    #[test]
    fn should_expand_azure_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let azure = find_config!(config.collectors, CollectorConfiguration::Azure);

        assert_eq!("azure", azure.id);
        assert_eq!("MyOrganization", azure.organization);
        assert_eq!("MyProject", azure.project);
        assert_eq!("MyDefinition_1", azure.definitions[0]);
        assert_eq!("MyDefinition_2", azure.definitions[1]);
        assert_eq!("MyBranch_1", azure.branches[0]);
        assert_eq!("MyBranch_2", azure.branches[1]);
        assert_eq!("SECRET-PAT-TOKEN", azure.get_pat());
    }

    #[test]
    fn should_expand_github_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let github = find_config!(config.collectors, CollectorConfiguration::GitHub);
        let (username, password) = github.get_credentials();

        assert_eq!("github", github.id);
        assert_eq!("spectresystems", github.owner);
        assert_eq!("duck", github.repository);
        assert_eq!("workflow.yml", github.workflow);
        assert_eq!("workflow.yml", github.workflow);
        assert_eq!("patrik", username);
        assert_eq!("hunter1!", password);
    }

    #[test]
    fn should_expand_octopus_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let octopus = find_config!(config.collectors, CollectorConfiguration::OctopusDeploy);

        assert_eq!("octopus", octopus.id);
        assert_eq!("https://localhost:9000", octopus.server_url);
        assert_eq!("SECRET-API-KEY", octopus.get_api_key());
        assert_eq!("Projects-1", octopus.projects[0].project_id);
        assert_eq!("Environments-1", octopus.projects[0].environments[0]);
        assert_eq!("Environments-2", octopus.projects[0].environments[1]);
    }

    #[test]
    fn should_expand_hue_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let observers = config.observers.as_ref().unwrap();
        let hue = find_config!(observers, ObserverConfiguration::Hue);

        assert_eq!("hue", hue.id);
        assert_eq!("https://192.168.1.155", hue.hub_url);
        assert_eq!(128, hue.brightness.unwrap());
        assert_eq!("patrik", hue.username);
        assert_eq!("Light_1", hue.lights[0]);
        assert_eq!("Light_2", hue.lights[1]);
    }

    #[test]
    fn should_expand_slack_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let observers = config.observers.as_ref().unwrap();
        let slack = find_config!(observers, ObserverConfiguration::Slack);

        assert_eq!("slack", slack.id);
        assert_eq!("https://example.com/Slack", slack.get_webhook_url());
    }

    #[test]
    fn should_expand_mattermost_configuration() {
        // Given, When
        let config = read_config!(CONFIGURATION);

        // Then
        let observers = config.observers.as_ref().unwrap();
        let mattermost = find_config!(observers, ObserverConfiguration::Mattermost);

        assert_eq!("mattermost", mattermost.id);
        assert_eq!("some-channel", &mattermost.channel.as_ref().unwrap()[..]);
        assert_eq!(
            "https://example.com/mattermost",
            mattermost.get_webhook_url()
        );
    }
}

#[cfg(test)]
mod utilities {
    use crate::config::*;

    impl TeamCityConfiguration {
        pub fn get_basic_auth(&self) -> (&str, &str) {
            match &self.credentials {
                TeamCityAuth::Guest => panic!("TeamCity configuration has guest credentials"),
                TeamCityAuth::BasicAuth { username, password } => (username, password),
            }
        }
    }

    impl AzureDevOpsConfiguration {
        pub fn get_pat(&self) -> &str {
            match &self.credentials {
                AzureDevOpsCredentials::Anonymous => {
                    panic!("Azure DevOps configuration have anonymous credentials")
                }
                AzureDevOpsCredentials::PersonalAccessToken(pat) => pat,
            }
        }
    }

    impl GitHubConfiguration {
        pub fn get_credentials(&self) -> (&str, &str) {
            match &self.credentials {
                GitHubCredentials::Basic { username, password } => (username, password),
            }
        }
    }

    impl OctopusDeployConfiguration {
        pub fn get_api_key(&self) -> &str {
            match &self.credentials {
                OctopusDeployCredentials::ApiKey(key) => key,
            }
        }
    }

    impl SlackConfiguration {
        pub fn get_webhook_url(&self) -> &str {
            match &self.credentials {
                SlackCredentials::Webhook { url } => url,
            }
        }
    }

    impl MattermostConfiguration {
        pub fn get_webhook_url(&self) -> &str {
            match &self.credentials {
                MattermostCredentials::Webhook { url } => url,
            }
        }
    }
}
