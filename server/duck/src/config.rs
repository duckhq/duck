use duck_server::DuckResult;
use duck_server::config::{ConfigurationHandle, Configuration};

pub struct FileConfiguration { }

impl ConfigurationHandle for FileConfiguration {
    fn has_changed(&self) -> DuckResult<bool> { 
        unimplemented!() 
    }

    /// Loads the configuration.
    fn load(&self) -> DuckResult<Configuration> { 
        unimplemented!() 
    }
}