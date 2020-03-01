use std::cell::Cell;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use duck_server::config::{Configuration, ConfigurationHandle};
use duck_server::DuckResult;

pub struct FileConfiguration<'a> {
    path: &'a PathBuf,
    reader: &'a dyn FileReader,
    modified: Cell<u64>,
}

trait FileReader {
    fn read(&self, path: &PathBuf) -> DuckResult<String>;
    fn modified(&self, path: &PathBuf) -> DuckResult<u64>;
}

impl<'a> FileConfiguration<'a> {
    pub fn create(path: &'a PathBuf) -> Self {
        FileConfiguration::new(path, &FileConfigurationReader {})
    }

    fn new(path: &'a PathBuf, reader: &'a dyn FileReader) -> Self {
        FileConfiguration {
            path,
            reader,
            modified: Cell::new(0),
        }
    }
}

struct FileConfigurationReader {}
impl FileReader for FileConfigurationReader {
    fn read(&self, path: &PathBuf) -> DuckResult<String> {
        Ok(fs::read_to_string(path)?)
    }

    fn modified(&self, path: &PathBuf) -> DuckResult<u64> {
        Ok(fs::metadata(path)?
            .modified()?
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs())
    }
}

impl<'a> ConfigurationHandle for FileConfiguration<'a> {
    fn has_changed(&self) -> DuckResult<bool> {
        let modified = self.reader.modified(self.path)?;
        if self.modified.get() != modified {
            return Ok(true);
        }
        Ok(false)
    }

    fn load(&self) -> DuckResult<Configuration> {
        // Read the configuration and deserialize it
        let json = self.reader.read(self.path)?;
        let config: Configuration = serde_json::from_str(&json[..])?;
        // Update the modified time to the current one.
        let modified = self.reader.modified(self.path)?;
        self.modified.set(modified);
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeFileReader {
        json: String,
        modified: Cell<u64>,
    }

    impl FakeFileReader {
        fn new<T: Into<String>>(json: T, modified: u64) -> Self {
            Self {
                json: json.into(),
                modified: Cell::new(modified),
            }
        }

        pub fn inc_modified(&self) {
            self.modified.set(self.modified.get() + 1);
        }
    }

    impl FileReader for FakeFileReader {
        fn read(&self, _path: &PathBuf) -> DuckResult<String> {
            Ok(self.json.clone())
        }
        
        fn modified(&self, _path: &PathBuf) -> DuckResult<u64> {
            Ok(self.modified.get())
        }
    }

    #[test]
    fn should_load_expected_configuration() {
        // Given
        let path = PathBuf::from("config.json");
        let reader = FakeFileReader::new(include_str!("test_data/config.json"), 1583092970);
        let config = FileConfiguration::new(&path, &reader);

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
        let handle = FileConfiguration::new(&path, &reader);

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
        let handle = FileConfiguration::new(&path, &reader);

        // When
        handle.load().unwrap();
        reader.inc_modified();
        let has_changed = handle.has_changed().unwrap();

        // Then
        assert!(has_changed);
    }
}
