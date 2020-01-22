use regex::Regex;
use std::collections::HashMap;

use log::warn;

use super::{CollectorConfiguration, Configuration, Validate};
use crate::utils::DuckResult;

impl Validate for Configuration {
    fn validate(&self) -> DuckResult<()> {
        if self.collectors.is_empty() {
            warn!("No collectors have been specified.");
        }

        validate_ids(&self)?;
        validate_collector_references(&self)?;

        Ok(())
    }
}

fn validate_ids(configuration: &Configuration) -> DuckResult<()> {
    // Make sure that all ids are unique and well formed.
    let mut unique_ids = std::collections::HashSet::<String>::new();
    let valid_id_pattern = Regex::new(r"^[a-zA-Z0-9_]+$")?;
    for id in configuration.get_all_ids() {
        if !valid_id_pattern.is_match(&id) {
            return Err(format_err!("The id '{}' is invalid.", id));
        }
        if unique_ids.contains(&id) {
            return Err(format_err!("Found duplicate id '{}' in configuration.", id));
        }
        unique_ids.insert(id);
    }

    Ok(())
}

fn validate_collector_references(configuration: &Configuration) -> DuckResult<()> {
    // Build a list of all collectors and whether or not they are enabled.
    let mut collectors: HashMap<String, bool> = HashMap::new();
    for collector in configuration.collectors.iter() {
        match collector {
            CollectorConfiguration::TeamCity(c) => {
                collectors.insert(
                    c.id.clone(),
                    match c.enabled {
                        None => true,
                        Some(enabled) => enabled,
                    },
                );
            }
            CollectorConfiguration::Azure(c) => {
                collectors.insert(
                    c.id.clone(),
                    match c.enabled {
                        None => true,
                        Some(enabled) => enabled,
                    },
                );
            }
            CollectorConfiguration::OctopusDeploy(c) => {
                collectors.insert(
                    c.id.clone(),
                    match c.enabled {
                        None => true,
                        Some(enabled) => enabled,
                    },
                );
            }
        }
    }

    // Validate referenced collectors.
    if let Some(observers) = &configuration.observers {
        for observer in observers.iter() {
            if let Some(references) = observer.get_collector_references() {
                for reference in references {
                    if !collectors.contains_key(&reference) {
                        // The referenced collector does not exist.
                        return Err(format_err!(
                            "The observer '{}' is dependent on collector '{}' which do not exist.",
                            observer.get_id(),
                            reference
                        ));
                    } else if observer.is_enabled() {
                        // Is the referenced collector disabled?
                        // This is not an error, but we should want about it.
                        if let Some(enabled) = collectors.get(&reference) {
                            if !enabled {
                                warn!(
                                    "The observer '{}' is dependent on disabled collector '{}'.",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Found duplicate id \\'foo\\' in configuration.")]
    fn should_return_error_if_two_collectors_have_the_same_id() {
        let config = Configuration::from_json(
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
    #[should_panic(expected = "Found duplicate id \\'foo\\' in configuration.")]
    fn should_return_error_if_a_collector_and_an_observer_have_the_same_id() {
        let config = Configuration::from_json(
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
        expected = "The observer \\'bar\\' is dependent on collector \\'baz\\' which do not exist."
    )]
    fn should_return_error_if_an_observer_is_dependent_on_non_existing_collector() {
        let config = Configuration::from_json(
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
