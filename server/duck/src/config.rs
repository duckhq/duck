use duck_server::DuckResult;
use duck_server::config::{ConfigurationHandle, Configuration};

pub struct FileConfiguration { }

impl ConfigurationHandle for FileConfiguration {
    fn reload() -> DuckResult<Option<Configuration>> { 
        unimplemented!() 
    }
}