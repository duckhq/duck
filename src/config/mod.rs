use std::path::PathBuf;

use serde::Deserialize;

use crate::utils::DuckResult;

mod validation;

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub interval: Option<Interval>,
    pub collectors: Vec<CollectorConfiguration>,
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

/// Timeout in seconds.
#[derive(Deserialize, Debug, Clone)]
pub struct Interval(pub u32);
impl Default for Interval {
    fn default() -> Self {
        Interval(15)
    }
}

#[derive(Deserialize, Clone)]
pub enum CollectorConfiguration {
    #[serde(rename = "teamcity")]
    TeamCity(TeamCityConfiguration),
    #[serde(rename = "azure")]
    Azure(AzureDevOpsConfiguration),
    #[serde(rename = "octopus")]
    OctopusDeploy(OctopusDeployConfiguration),
}

#[derive(Deserialize, Clone)]
pub struct TeamCityConfiguration {
    pub id: String,
    pub enabled: Option<bool>,
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    pub credentials: TeamCityAuth,
    pub builds: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub enum TeamCityAuth {
    #[serde(rename = "guest")]
    Guest,
    #[serde(rename = "basic")]
    BasicAuth { username: String, password: String },
}

#[derive(Deserialize, Clone)]
pub struct AzureDevOpsConfiguration {
    pub id: String,
    pub enabled: Option<bool>,
    pub organization: String,
    pub project: String,
    pub credentials: AzureDevOpsCredentials,
    pub branches: Vec<String>,
    pub definitions: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub enum AzureDevOpsCredentials {
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "pat")]
    PersonalAccessToken(String),
}

#[derive(Deserialize, Clone)]
pub struct OctopusDeployConfiguration {
    pub id: String,
    pub enabled: Option<bool>,
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    pub credentials: OctopusDeployCredentials,
    pub projects: Vec<OctopusDeployProject>,
}

#[derive(Deserialize, Clone)]
pub struct OctopusDeployProject {
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub environments: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub enum OctopusDeployCredentials {
    #[serde(rename = "apiKey")]
    ApiKey(String),
}

#[derive(Deserialize, Clone)]
pub enum ObserverConfiguration {
    #[serde(rename = "hue")]
    Hue(HueConfiguration),
    #[serde(rename = "slack")]
    Slack(SlackConfiguration),
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

#[derive(Deserialize, Clone)]
pub struct HueConfiguration {
    pub id: String,
    pub enabled: Option<bool>,
    pub collectors: Option<Vec<String>>,
    pub brightness: Option<u8>,
    #[serde(rename = "hubUrl")]
    pub hub_url: String,
    pub username: String,
    pub lights: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct SlackConfiguration {
    pub id: String,
    pub enabled: Option<bool>,
    pub collectors: Option<Vec<String>>,
    pub credentials: SlackCredentials,
    pub channel: Option<String>,
}

#[derive(Deserialize, Clone)]
pub enum SlackCredentials {
    #[serde(rename = "webhook")]
    Webhook { url: String },
}

#[derive(Deserialize, Clone)]
pub struct MattermostConfiguration {
    pub id: String,
    pub enabled: Option<bool>,
    pub collectors: Option<Vec<String>>,
    pub channel: Option<String>,
    pub credentials: MattermostCredentials,
}

#[derive(Deserialize, Clone)]
pub enum MattermostCredentials {
    #[serde(rename = "webhook")]
    Webhook { url: String },
}
