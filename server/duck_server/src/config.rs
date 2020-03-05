use crate::DuckResult;

/// Represents a way of loading a configuration
pub trait ConfigurationLoader: Sync + Send + Clone {
    fn exist(&self) -> bool;
    fn has_changed(&self) -> DuckResult<bool>;
    fn load(&self) -> DuckResult<Configuration>;
}

/// Represents a Duck configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct Configuration {
    /// # Duck frontend title
    /// The title that is displayed in the UI
    #[serde(default = "default_title")]
    pub title: String,
    /// # Update interval
    /// The update interval in seconds
    #[serde(default = "default_interval")]
    pub interval: u16,
    /// # Collectors
    #[serde(default)]
    pub collectors: Vec<CollectorConfiguration>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum CollectorConfiguration {
    /// # TeamCity collector
    /// Gets builds from TeamCity
    #[serde(rename = "teamcity")]
    TeamCity(TeamCityConfiguration),
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[cfg(test)]
impl Default for Configuration {
    fn default() -> Self {
        serde_json::from_str("{}").unwrap()
    }
}

///////////////////////////////////////////////////////////
// Default values

fn default_title() -> String {
    "Duck".to_owned()
}

fn default_interval() -> u16 {
    15
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn should_assign_default_values_to_deserialized_configuration() {
        // Given, When
        let config: Configuration = serde_json::from_str("{}").unwrap();

        // Then
        assert_eq!("Duck", config.title);
        assert_eq!(15, config.interval);
    }
}
