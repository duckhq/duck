use schemars::JsonSchema;
use serde::Deserialize;

use crate::utils::text::Expander;
use crate::utils::text::VariableProvider;
use crate::DuckResult;

mod expansions;
mod validation;

pub mod loader;

pub trait Validate {
    fn validate(&self) -> DuckResult<()>;
}

/// Represents a way of loading a configuration
pub trait ConfigurationLoader: Sync + Send + Clone {
    fn exist(&self) -> bool;
    fn has_changed(&self) -> DuckResult<bool>;
    fn load(&self, variables: &dyn VariableProvider) -> DuckResult<Configuration>;
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Default)]
pub struct Configuration {
    /// # Update interval
    /// The update interval in seconds
    #[serde(default = "default_interval")]
    pub interval: u16,
    /// # Views
    pub views: Option<Vec<ViewConfiguration>>,
    /// # Duck frontend title
    /// The title that is displayed in the UI
    #[serde(default = "default_title")]
    pub title: String,
    /// # Collectors
    pub collectors: Vec<CollectorConfiguration>,
    /// # Observers
    #[serde(default)]
    pub observers: Option<Vec<ObserverConfiguration>>,
}

impl Configuration {
    pub fn from_json<T: Into<String>>(
        variables: &dyn VariableProvider,
        json: T,
    ) -> DuckResult<Self> {
        let expander = &Expander::new(variables);
        let json = expander.expand(json)?;
        let config: Configuration = serde_json::from_str(&json[..])?;
        config.validate()?;
        Ok(config)
    }

    pub fn get_all_ids(&self) -> Vec<String> {
        // Get all collector id:s
        let mut result: Vec<String> = self
            .collectors
            .iter()
            .map(|i| i.get_id().to_owned())
            .collect();
        // Get all observer id:s
        match self.observers {
            Option::None => (),
            Option::Some(ref observers) => {
                for observer in observers.iter() {
                    result.push(observer.get_id().to_owned());
                }
            }
        }
        result
    }

    pub fn collector_exist(&self, id: &str) -> bool {
        for collector in self.collectors.iter() {
            if collector.get_id() == id {
                return true;
            }
        }
        false
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct Interval(pub u32);
impl Default for Interval {
    fn default() -> Self {
        Interval(15)
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone)]
pub struct ViewConfiguration {
    /// # View ID
    /// The ID of the view
    pub id: String,
    /// # View name
    /// the name of the view
    pub name: String,
    /// # Included collectors
    /// The collectors included in this view
    pub collectors: Vec<String>,
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
    /// # GitHub collector
    /// Gets builds from GitHub Actions
    #[serde(rename = "github")]
    GitHub(GitHubConfiguration),
    /// # Octopus Deploy collector
    /// Gets deployments from Octopus Deploy
    #[serde(rename = "octopus")]
    OctopusDeploy(OctopusDeployConfiguration),
    /// # AppVeyor collector
    /// Gets builds from AppVeyor
    #[serde(rename = "appveyor")]
    AppVeyor(AppVeyorConfiguration),
    /// # Duck collector
    /// Gets builds from another Duck instance
    #[serde(rename = "duck")]
    Duck(DuckConfiguration),
    /// # Debug collector
    /// Gets builds from a Duck debugger instance
    #[serde(rename = "debugger")]
    Debugger(DebuggerConfiguration),
}

impl CollectorConfiguration {
    pub fn get_id(&self) -> &str {
        match self {
            CollectorConfiguration::TeamCity(c) => &c.id,
            CollectorConfiguration::Azure(c) => &c.id,
            CollectorConfiguration::GitHub(c) => &c.id,
            CollectorConfiguration::OctopusDeploy(c) => &c.id,
            CollectorConfiguration::AppVeyor(c) => &c.id,
            CollectorConfiguration::Duck(c) => &c.id,
            CollectorConfiguration::Debugger(c) => &c.id,
        }
    }

    pub fn is_enabled(&self) -> bool {
        if let Some(enabled) = match self {
            CollectorConfiguration::TeamCity(c) => c.enabled,
            CollectorConfiguration::Azure(c) => c.enabled,
            CollectorConfiguration::GitHub(c) => c.enabled,
            CollectorConfiguration::OctopusDeploy(c) => c.enabled,
            CollectorConfiguration::AppVeyor(c) => c.enabled,
            CollectorConfiguration::Duck(c) => c.enabled,
            CollectorConfiguration::Debugger(c) => c.enabled,
        } {
            return enabled;
        }
        return true;
    }
}

impl Validate for CollectorConfiguration {
    fn validate(&self) -> DuckResult<()> {
        match self {
            CollectorConfiguration::TeamCity(c) => c.validate(),
            CollectorConfiguration::Azure(c) => c.validate(),
            CollectorConfiguration::GitHub(c) => c.validate(),
            CollectorConfiguration::OctopusDeploy(c) => c.validate(),
            CollectorConfiguration::AppVeyor(c) => c.validate(),
            CollectorConfiguration::Duck(c) => c.validate(),
            CollectorConfiguration::Debugger(c) => c.validate(),
        }
    }
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

impl Validate for ObserverConfiguration {
    fn validate(&self) -> DuckResult<()> {
        match self {
            ObserverConfiguration::Hue(c) => c.validate(),
            ObserverConfiguration::Slack(c) => c.validate(),
            ObserverConfiguration::Mattermost(c) => c.validate(),
        }
    }
}

///////////////////////////////////////////////////////////
// AppVeyor

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AppVeyorConfiguration {
    /// # The AppVeyor collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The TeamCity credentials
    pub credentials: AppVeyorCredentials,
    /// # The AppVeyor account
    pub account: String,
    /// # The AppVeyor project
    pub project: String,
    /// # The number of builds to retrieve
    #[serde(default)]
    pub count: Option<u16>,
}

impl AppVeyorConfiguration {
    pub fn get_count(&self) -> u16 {
        match self.count {
            None => 1,
            Some(count) => std::cmp::max(1, count),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum AppVeyorCredentials {
    #[serde(rename = "bearer")]
    Bearer(String),
}

///////////////////////////////////////////////////////////
// TeamCity

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
    BasicAuth {
        /// # The username to use
        username: String,
        /// # The password to use
        password: String,
    },
}

///////////////////////////////////////////////////////////
// Azure DevOps

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct AzureDevOpsConfiguration {
    /// # The Azure DevOps collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The Azure DevOps server URL.
    /// Only required if Duck should collect builds
    /// from a self-hosted instance of Azure DevOps Server.
    #[serde(rename = "serverUrl")]
    pub server_url: Option<String>,
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

///////////////////////////////////////////////////////////
// GitHub Actions

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct GitHubConfiguration {
    /// # The GitHub collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The GitHub owner
    pub owner: String,
    /// # The GitHub repository
    pub repository: String,
    /// # The GitHub credentials
    pub credentials: GitHubCredentials,
    /// # The GitHub Actions workflow
    pub workflow: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum GitHubCredentials {
    /// # Basic authentication
    /// Authenticate using basic authentication
    #[serde(rename = "basic")]
    Basic {
        /// # The username to use
        username: String,
        /// # The password to use
        password: String,
    },
}

///////////////////////////////////////////////////////////
// Octopus Deploy

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

///////////////////////////////////////////////////////////
// Debugger

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct DebuggerConfiguration {
    /// # The Duck debugger collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The Duck debugger URL
    #[serde(rename = "serverUrl")]
    pub server_url: String,
}

///////////////////////////////////////////////////////////
// Duck

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct DuckConfiguration {
    /// # The Duck collector ID
    pub id: String,
    /// # Determines whether or not this collector is enabled
    #[serde(default)]
    pub enabled: Option<bool>,
    /// # The Duck server URL
    #[serde(rename = "serverUrl")]
    pub server_url: String,
    /// # The view to get builds from
    #[serde(default)]
    pub view: Option<String>,
}

///////////////////////////////////////////////////////////
// Hue

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
    /// # An optional filter expression
    pub filter: Option<String>,
}

///////////////////////////////////////////////////////////
// Slack

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
    /// # An optional filter expression
    pub filter: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum SlackCredentials {
    /// # Webhook
    /// Send messages directly to a webhook
    #[serde(rename = "webhook")]
    Webhook { url: String },
}

///////////////////////////////////////////////////////////
// Mattermost

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
    /// # An optional filter expression
    pub filter: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub enum MattermostCredentials {
    /// # Webhook
    /// Send messages directly to a webhook
    #[serde(rename = "webhook")]
    Webhook { url: String },
}

///////////////////////////////////////////////////////////
// Default values

fn default_title() -> String {
    "Duck".to_owned()
}

fn default_interval() -> u16 {
    15
}
