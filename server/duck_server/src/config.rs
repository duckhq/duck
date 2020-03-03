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
