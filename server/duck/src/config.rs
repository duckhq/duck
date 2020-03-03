use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use duck_server::config::{Configuration, ConfigurationLoader};
use duck_server::DuckResult;

///////////////////////////////////////////////////////////
// Configuration loader

#[derive(Clone)]
pub struct JsonConfigurationLoader<'a> {
    path: PathBuf,
    reader: &'a dyn FileReader,
    modified: Arc<Mutex<u64>>,
}

impl<'a> JsonConfigurationLoader<'a> {
    pub fn create(path: PathBuf) -> Self {
        JsonConfigurationLoader::new(path, &DefaultFileReader {})
    }

    fn new(path: PathBuf, reader: &'a dyn FileReader) -> Self {
        JsonConfigurationLoader {
            path,
            reader,
            modified: Arc::new(Mutex::new(0)),
        }
    }
}

impl<'a> ConfigurationLoader for JsonConfigurationLoader<'a> {
    fn exist(&self) -> bool { 
        self.path.exists()
    }
    
    fn has_changed(&self) -> DuckResult<bool> {
        let modified = self.reader.modified(&self.path)?;
        if *self.modified.lock().unwrap() != modified {
            return Ok(true);
        }
        Ok(false)
    }

    fn load(&self) -> DuckResult<Configuration> {
        // Read the configuration and deserialize it
        let json = self.reader.read_to_string(&self.path)?;
        let config: Configuration = serde_json::from_str(&json[..])?;
        // Update the modified time to the current one.
        let modified = self.reader.modified(&self.path)?;
        *self.modified.lock().unwrap() = modified;
        Ok(config)
    }
}

///////////////////////////////////////////////////////////
// File reader

trait FileReader: Send + Sync {
    /// Returns the content of the file as a string
    fn read_to_string(&self, path: &PathBuf) -> DuckResult<String>;
    /// Gets the modified time as Epoch time
    fn modified(&self, path: &PathBuf) -> DuckResult<u64>;
}

struct DefaultFileReader {}
impl FileReader for DefaultFileReader {
    fn read_to_string(&self, path: &PathBuf) -> DuckResult<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn modified(&self, path: &PathBuf) -> DuckResult<u64> {
        Ok(fs::metadata(path)?
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs())
    }
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeFileReader {
        json: String,
        modified: Arc<Mutex<u64>>,
    }

    impl FakeFileReader {
        fn new<T: Into<String>>(json: T, modified: u64) -> Self {
            Self {
                json: json.into(),
                modified: Arc::new(Mutex::new(modified)),
            }
        }

        pub fn inc_modified(&self) {
            let mut modified = self.modified.lock().unwrap();
            *modified = *modified + 1;
        }
    }

    impl FileReader for FakeFileReader {
        fn read_to_string(&self, _path: &PathBuf) -> DuckResult<String> {
            Ok(self.json.clone())
        }

        fn modified(&self, _path: &PathBuf) -> DuckResult<u64> {
            let modified = self.modified.lock().unwrap();
            Ok(*modified)
        }
    }

    #[test]
    fn should_load_expected_configuration() {
        // Given
        let path = PathBuf::from("config.json");
        let reader = FakeFileReader::new(include_str!("test_data/config.json"), 1583092970);
        let config = JsonConfigurationLoader::new(path, &reader);

        // When
        let config = config.load().unwrap();

        // Then
        assert_eq!(99, config.interval);
        assert_eq!("Duck test server", config.title);
    }

    #[test]
    fn should_indicate_if_configuration_has_not_changed_since_read() {
        // Given
        let path = PathBuf::from("config.json");
        let reader = FakeFileReader::new(include_str!("test_data/config.json"), 1583092970);
        let handle = JsonConfigurationLoader::new(path, &reader);

        // When
        handle.load().unwrap();
        let has_changed = handle.has_changed().unwrap();

        // Then
        assert!(!has_changed);
    }

    #[test]
    fn should_indicate_if_configuration_changed_since_read() {
        // Given
        let path = PathBuf::from("config.json");
        let reader = FakeFileReader::new(include_str!("test_data/config.json"), 1583092970);
        let handle = JsonConfigurationLoader::new(path, &reader);

        // When
        handle.load().unwrap();
        reader.inc_modified();
        let has_changed = handle.has_changed().unwrap();

        // Then
        assert!(has_changed);
    }
}
