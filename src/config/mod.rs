use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::utils::DuckResult;

mod validation;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct Configuration {
    /// # Update interval
    /// The update interval in seconds
    #[serde(default)]
    pub interval: Option<Interval>,
    /// # Duck frontend title
    /// The title that is displayed in the UI
    #[serde(default)]
    pub title: Option<String>,
    /// # Collectors
    pub collectors: Vec<CollectorConfiguration>,
    /// # Observers
    #[serde(default)]
    pub observers: Option<Vec<ObserverConfiguration>>,
}

pub trait Validate {
    fn validate(&self) -> DuckResult<()>;
}

impl Configuration {
    #[allow(dead_code)]
    pub fn from_json<T: Into<String>>(json: T) -> DuckResult<Self> {
        let config: Configuration = serde_json::from_str(&json.into()[..])?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_file(path: PathBuf) -> DuckResult<Self> {
        let json = std::fs::read_to_string(path)?;
        let config: Configuration = serde_json::from_str(&json[..])?;
        config.validate()?;
        Ok(config)
    }

    pub fn get_title(&self) -> &str {
        match &self.title {
            Some(title) => title,
            None => "Duck",
        }
    }

    pub fn get_interval(&self) -> u64 {
        if let Some(i) = &self.interval {
            if i.0 >= 15 {
                return u64::from(i.0);
            }
        }
        return 15;
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        // Get all collector id:s
        let mut result: Vec<String> = self
            .collectors
            .iter()
            .map(|i| match i {
                CollectorConfiguration::TeamCity(c) => c.id.clone(),
                CollectorConfiguration::Azure(c) => c.id.clone(),
                CollectorConfiguration::OctopusDeploy(c) => c.id.clone(),
            })
            .collect();
        // Get all observer id:s
        match self.observers {
            Option::None => (),
            Option::Some(ref observers) => {
                for observer in observers.iter() {
                    match observer {
                        ObserverConfiguration::Hue(c) => result.push(c.id.clone()),
                        ObserverConfiguration::Slack(c) => result.push(c.id.clone()),
                        ObserverConfiguration::Mattermost(c) => result.push(c.id.clone()),
                    };
                }
            }
        }
        result
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Interval(pub u32);
impl Default for Interval {
    fn default() -> Self {
        Interval(15)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum CollectorConfiguration {
    /// # TeamCity collector
    /// Gets builds from TeamCity
    #[serde(rename = "teamcity")]
    TeamCity(TeamCityConfiguration),
    /// # Azure DevOps collector
    /// Gets builds from Azure DevOps
    #[serde(rename = "azure")]
    Azure(AzureDevOpsConfiguration),
    /// # Octopus Deploy collector
    /// Gets deployments from Octopus Deploy
    #[serde(rename = "octopus")]
    OctopusDeploy(OctopusDeployConfiguration),
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct TeamCityConfiguration {
    /// # The TeamCity collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The TeamCity server URL
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    /// # The TeamCity credentials
    pub credentials: TeamCityAuth,
    /// # The TeamCity builds definitions to include
    pub builds: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum TeamCityAuth {
    /// # Guest
    /// Authenticate as guest
    #[serde(rename = "guest")]
    Guest,
    /// # Basic authentication
    /// Authenticate using basic authentication
    #[serde(rename = "basic")]
    BasicAuth { username: String, password: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AzureDevOpsConfiguration {
    /// # The Azure DevOps collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The Azure DevOps organization
    pub organization: String,
    /// # The Azure DevOps project
    pub project: String,
    /// # The Azure DevOps credentials
    pub credentials: AzureDevOpsCredentials,
    /// # The branches to include
    pub branches: Vec<String>,
    /// # The build definitions to include
    pub definitions: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum AzureDevOpsCredentials {
    /// # Anonymous
    /// Use anonymous authentication
    #[serde(rename = "anonymous")]
    Anonymous,
    /// # Personal access token
    /// Authenticate using a personal access token (PAT)
    #[serde(rename = "pat")]
    PersonalAccessToken(String),
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct OctopusDeployConfiguration {
    /// # The Octopus Deploy collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The Octopus Deploy server URL
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    /// # The Octopus Deploy credentials
    pub credentials: OctopusDeployCredentials,
    /// # The Octopus Deploy projects to include
    pub projects: Vec<OctopusDeployProject>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct OctopusDeployProject {
    /// # The Octopus Deploy project ID
    #[serde(rename = "projectId")]
    pub project_id: String,
    /// # The Octopus Deploy environment IDs within the project
    pub environments: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum OctopusDeployCredentials {
    /// # API Key
    /// Authenticate using an API key
    #[serde(rename = "apiKey")]
    ApiKey(String),
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum ObserverConfiguration {
    /// # Philips Hue observer
    #[serde(rename = "hue")]
    Hue(HueConfiguration),
    /// # Slack observer
    #[serde(rename = "slack")]
    Slack(SlackConfiguration),
    /// # Mattermost observer
    #[serde(rename = "mattermost")]
    Mattermost(MattermostConfiguration),
}

impl ObserverConfiguration {
    pub fn get_id(&self) -> &str {
        match self {
            ObserverConfiguration::Hue(c) => &c.id,
            ObserverConfiguration::Slack(c) => &c.id,
            ObserverConfiguration::Mattermost(c) => &c.id,
        }
    }

    pub fn is_enabled(&self) -> bool {
        if let Some(enabled) = match self {
            ObserverConfiguration::Hue(c) => c.enabled,
            ObserverConfiguration::Slack(c) => c.enabled,
            ObserverConfiguration::Mattermost(c) => c.enabled,
        } {
            return enabled;
        }
        return true;
    }

    pub fn get_collector_references(&self) -> Option<Vec<String>> {
        match self {
            ObserverConfiguration::Hue(c) => c.collectors.clone(),
            ObserverConfiguration::Slack(c) => c.collectors.clone(),
            ObserverConfiguration::Mattermost(c) => c.collectors.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct HueConfiguration {
    /// # The Philips Hue collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The collectors to include events from
    #[serde(default)]
    pub collectors: Option<Vec<String>>,
    /// # The brightness of the lamps
    #[serde(default)]
    pub brightness: Option<u8>,
    /// # The Philips Hue hub URL
    #[serde(rename = "hubUrl")]
    pub hub_url: String,
    /// # The Philips Hue username
    pub username: String,
    /// # The lights that should be controlled by this observer
    pub lights: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct SlackConfiguration {
    /// # The Slack collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The collectors to include events from
    #[serde(default)]
    pub collectors: Option<Vec<String>>,
    /// # The Slack credentials
    pub credentials: SlackCredentials,
    /// # The Slack channel to send messages to
    #[serde(default)]
    pub channel: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum SlackCredentials {
    /// # Webhook
    /// Send messages directly to a webhook
    #[serde(rename = "webhook")]
    Webhook { url: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct MattermostConfiguration {
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The collectors to include events from
    #[serde(default)]
    pub collectors: Option<Vec<String>>,
    /// # The Mattermost channel to send messages to
    #[serde(default)]
    pub channel: Option<String>,
    /// # The Mattermost credentials
    pub credentials: MattermostCredentials,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum MattermostCredentials {
    /// # Webhook
    /// Send messages directly to a webhook
    #[serde(rename = "webhook")]
    Webhook { url: String },
}
