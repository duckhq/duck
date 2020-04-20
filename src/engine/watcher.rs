use crate::config::{Configuration, ConfigurationLoader};
use crate::utils::text::VariableProvider;
use log::{debug, error, warn};

///////////////////////////////////////////////////////////
// State

#[derive(PartialEq)]
enum State {
    Started,
    Loaded,
    Error(WatchError),
}

#[derive(PartialEq)]
enum WatchError {
    NotFound,
    CheckError,
    LoadError,
}

///////////////////////////////////////////////////////////
// Context

pub struct Context {
    state: State,
    variables: Box<dyn VariableProvider>,
}

impl Context {
    pub fn new<T: VariableProvider + Sized + 'static>(variables: T) -> Self {
        Self {
            state: State::Started,
            variables: Box::new(variables),
        }
    }
}

impl Context {
    fn has_error(&self, error: WatchError) -> bool {
        match &self.state {
            State::Error(e) => e == &error,
            _ => false,
        }
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }
}

///////////////////////////////////////////////////////////
// Execution

pub fn try_load(context: &mut Context, loader: &impl ConfigurationLoader) -> Option<Configuration> {
    if loader.exist() {
        match loader.has_changed() {
            Ok(has_changed) => {
                if !has_changed {
                    return None;
                }

                match loader.load(&*context.variables) {
                    Ok(config) => {
                        debug!("Configuration file loaded");
                        context.set_state(State::Loaded);
                        return Some(config);
                    }
                    Err(err) => {
                        if !context.has_error(WatchError::LoadError) {
                            error!("Could not load configuration file: {}", err);
                            context.set_state(State::Error(WatchError::LoadError))
                        }
                    }
                };
            }
            Err(err) => {
                if !context.has_error(WatchError::CheckError) {
                    error!("Could not check configuration file: {}", err);
                    context.set_state(State::Error(WatchError::CheckError));
                }
            }
        };
    } else if !context.has_error(WatchError::NotFound) {
        warn!("Configuration file could not be found.");
        context.set_state(State::Error(WatchError::NotFound));
    }

    None
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::text::TestVariableProvider;

    #[derive(Clone)]
    struct LoadDefaultConfiguration {}
    impl ConfigurationLoader for LoadDefaultConfiguration {
        fn exist(&self) -> bool {
            true
        }
        fn has_changed(&self) -> crate::DuckResult<bool> {
            Ok(true)
        }
        fn load(&self, _: &dyn VariableProvider) -> crate::DuckResult<Configuration> {
            Ok(Configuration::default())
        }
    }

    #[derive(Clone)]
    struct FailToLoad {}
    impl ConfigurationLoader for FailToLoad {
        fn exist(&self) -> bool {
            true
        }
        fn has_changed(&self) -> crate::DuckResult<bool> {
            Ok(true)
        }
        fn load(&self, _: &dyn VariableProvider) -> crate::DuckResult<Configuration> {
            Err(format_err!("Oh noes!"))
        }
    }

    #[derive(Clone)]
    struct FailToCheck {}
    impl ConfigurationLoader for FailToCheck {
        fn exist(&self) -> bool {
            true
        }
        fn has_changed(&self) -> crate::DuckResult<bool> {
            Err(format_err!("Oh noes!"))
        }
        fn load(&self, _: &dyn VariableProvider) -> crate::DuckResult<Configuration> {
            unimplemented!()
        }
    }

    #[derive(Clone)]
    struct NotFound {}
    impl ConfigurationLoader for NotFound {
        fn exist(&self) -> bool {
            false
        }
        fn has_changed(&self) -> crate::DuckResult<bool> {
            unimplemented!()
        }
        fn load(&self, _: &dyn VariableProvider) -> crate::DuckResult<Configuration> {
            unimplemented!()
        }
    }

    #[test]
    pub fn should_set_state_to_loaded_if_configuration_was_loaded() {
        // Given
        let mut context = Context::new(TestVariableProvider::new());
        let loader = LoadDefaultConfiguration {};

        // When
        let result = try_load(&mut context, &loader);

        // Then
        assert!(result.is_some(), "Did not get a configuration back");
        assert!(context.state == State::Loaded, "State was not 'Loaded'");
    }

    #[test]
    pub fn should_set_state_to_load_error_if_loading_configuration_failed() {
        // Given
        let mut context = Context::new(TestVariableProvider::new());
        let loader = FailToLoad {};

        // When
        let result = try_load(&mut context, &loader);

        // Then
        assert!(result.is_none(), "Got a configuration back");
        assert!(
            context.has_error(WatchError::LoadError),
            "Configuration did not fail to load"
        );
    }

    #[test]
    pub fn should_set_state_to_check_error_if_checking_for_changes_failed() {
        // Given
        let mut context = Context::new(TestVariableProvider::new());
        let loader = FailToCheck {};

        // When
        let result = try_load(&mut context, &loader);

        // Then
        assert!(result.is_none(), "Got a configuration back");
        assert!(
            context.has_error(WatchError::CheckError),
            "Checking for changes did not fail"
        );
    }

    #[test]
    pub fn should_set_state_to_not_found_if_configuration_does_not_exist() {
        // Given
        let mut context = Context::new(TestVariableProvider::new());
        let loader = NotFound {};

        // When
        let result = try_load(&mut context, &loader);

        // Then
        assert!(result.is_none(), "Got a configuration back");
        assert!(
            context.has_error(WatchError::NotFound),
            "Configuration file was found"
        );
    }
}
