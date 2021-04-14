use regex::Regex;
use std::collections::{HashMap, HashSet};

use log::warn;

use super::{Configuration, Validate};
use crate::DuckResult;

pub const ID_PATTERN: &str = r"^[a-zA-Z0-9_\-.]+$";

impl Validate for Configuration {
    fn validate(&self) -> DuckResult<()> {
        if self.collectors.is_empty() {
            warn!("No collectors have been specified");
        }

        validate_views(&self)?;
        validate_ids(&self)?;
        validate_collector_references(&self)?;

        // Validate collectors
        for collector in self.collectors.iter() {
            collector.validate()?;
        }

        // Validate observers
        if let Some(observers) = &self.observers {
            for observer in observers.iter() {
                observer.validate()?;
            }
        }

        Ok(())
    }
}

fn validate_views(configuration: &Configuration) -> DuckResult<()> {
    let valid_id_pattern = Regex::new(ID_PATTERN)?;
    if let Some(views) = &configuration.views {
        let mut known_ids = HashSet::<String>::new();
        for view in views.iter() {
            if !valid_id_pattern.is_match(&view.id) {
                return Err(format_err!("The view id '{}' is invalid", view.id));
            }
            if known_ids.contains(&view.id) {
                return Err(format_err!(
                    "Found duplicate view id '{}' in configuration",
                    view.id
                ));
            }
            for view_collector in view.collectors.iter() {
                if !configuration.collector_exist(&view_collector) {
                    return Err(format_err!(
                        "The view '{}' depends on collector '{}' which does not exist",
                        view.id,
                        view_collector
                    ));
                }
            }
            known_ids.insert(view.id.clone());
        }
    };

    Ok(())
}

fn validate_ids(configuration: &Configuration) -> DuckResult<()> {
    // Make sure that all ids are unique and well formed.
    let mut unique_ids = std::collections::HashSet::<String>::new();
    let valid_id_pattern = Regex::new(ID_PATTERN)?;
    for id in configuration.get_all_ids() {
        if !valid_id_pattern.is_match(&id) {
            return Err(format_err!("The id '{}' is invalid", id));
        }
        if unique_ids.contains(&id) {
            return Err(format_err!("Found duplicate id '{}' in configuration", id));
        }
        unique_ids.insert(id);
    }

    Ok(())
}

fn validate_collector_references(configuration: &Configuration) -> DuckResult<()> {
    // Build a list of all collectors and whether or not they are enabled.
    let mut collectors: HashMap<String, bool> = HashMap::new();
    for collector in configuration.collectors.iter() {
        collectors.insert(collector.get_id().to_string(), collector.is_enabled());
    }

    // Validate referenced collectors.
    if let Some(observers) = &configuration.observers {
        for observer in observers.iter() {
            if let Some(references) = observer.get_collector_references() {
                for reference in references {
                    if !collectors.contains_key(&reference) {
                        // The referenced collector does not exist.
                        return Err(format_err!(
                            "The observer '{}' is dependent on collector '{}' which do not exist",
                            observer.get_id(),
                            reference
                        ));
                    } else if observer.is_enabled() {
                        // Is the referenced collector disabled?
                        // This is not an error, but we should want about it.
                        if let Some(enabled) = collectors.get(&reference) {
                            if !enabled {
                                warn!(
                                    "The observer '{}' is dependent on disabled collector '{}'",
                                    observer.get_id(),
                                    reference
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

///////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::text::TestVariableProvider;

    #[test]
    #[should_panic(expected = "Found duplicate view id \\'foo\\' in configuration")]
    fn should_return_error_if_views_have_the_same_id() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "views": [
                    {
                        "id": "foo",
                        "name": "Foo",
                        "collectors": [ ]
                    },
                    {
                        "id": "foo",
                        "name": "Bar",
                        "collectors": [ ]
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "The view id \\'foo bar\\' is invalid")]
    fn should_return_error_if_a_view_have_an_invalid_id() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "views": [
                    {
                        "id": "foo bar",
                        "name": "Foo",
                        "collectors": [ ]
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(
        expected = "The view \\'foo\\' depends on collector \\'bar\\' which does not exist"
    )]
    fn should_return_error_if_a_view_depends_on_collector_that_does_not_exist() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ ],
                "views": [
                    {
                        "id": "foo",
                        "name": "Foo",
                        "collectors": [ "bar" ]
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "Found duplicate id \\'foo\\' in configuration")]
    fn should_return_error_if_two_collectors_have_the_same_id() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "https://build1.example.com",
                            "credentials": "guest",
                            "builds": [ "Foo" ]
                        }
                    },
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "https://build2.example.com",
                            "credentials": "guest",
                            "builds": [ "Bar" ]
                        }
                    }
                ] 
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "Found duplicate id \\'foo\\' in configuration")]
    fn should_return_error_if_a_collector_and_an_observer_have_the_same_id() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "https://build1.example.com",
                            "credentials": "guest",
                            "builds": [ "Foo" ]
                        }
                    }
                ] ,
                "observers": [
                    {
                        "hue": {
                            "id": "foo",
                            "hubUrl": "https://localhost:5000",
                            "username": "SOME-SECRET-USERNAME",
                            "lights": [ "3" ]
                        }
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(
        expected = "The observer \\'bar\\' is dependent on collector \\'baz\\' which do not exist"
    )]
    fn should_return_error_if_an_observer_is_dependent_on_non_existing_collector() {
        let config = Configuration::from_json(
            &TestVariableProvider::new(),
            r#"
            { 
                "collectors": [ 
                    {
                        "teamcity": {
                            "id": "foo",
                            "serverUrl": "https://build1.example.com",
                            "credentials": "guest",
                            "builds": [ "Foo" ]
                        }
                    }
                ] ,
                "observers": [
                    {
                        "hue": {
                            "id": "bar",
                            "collectors": [ "baz" ],
                            "hubUrl": "https://localhost:5000",
                            "username": "SOME-SECRET-USERNAME",
                            "lights": [ "3" ]
                        }
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }
}
