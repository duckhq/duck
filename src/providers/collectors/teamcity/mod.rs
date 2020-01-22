use std::sync::Arc;

use log::{error, trace, warn};
use waithandle::{EventWaitHandle, WaitHandle};

use crate::builds::{Build, BuildProvider};
use crate::config::TeamCityConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo};
use crate::utils::{date, DuckResult};

use self::client::TeamCityClient;

mod client;
mod validation;

pub struct TeamCityCollector {
    client: TeamCityClient,
    build_types: Vec<String>,
    info: CollectorInfo,
}

impl TeamCityCollector {
    pub fn new(config: &TeamCityConfiguration) -> Self {
        return Self {
            client: TeamCityClient::new(config),
            build_types: config.builds.clone(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: BuildProvider::TeamCity,
            },
        };
    }
}

impl Collector for TeamCityCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        handle: Arc<EventWaitHandle>,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        // Make sure TeamCity is online.
        if !self.client.is_online() {
            error!("There was a problem contacting TeamCity.");
            return Err(format_err!("There was a problem contacting TeamCity."));
        }

        // Get all known build types from TeamCity.
        let known_build_types = self.client.get_build_types()?;

        // Get builds for all build types.
        for build_type in self.build_types.iter() {
            if handle.check().unwrap() {
                return Ok(());
            }

            // Make sure the build type is known.
            let found = match known_build_types.iter().find(|t| t.id.eq(build_type)) {
                Option::None => {
                    warn!(
                        "The build type '{}' does not exist in TeamCity.",
                        build_type
                    );
                    continue;
                }
                Option::Some(r) => r,
            };

            trace!("Getting builds for {}...", build_type);
            let result = self.client.get_builds(found)?;
            for branch in result.branches {
                for build in branch.builds.builds {
                    callback(Build::new(
                        build.id.to_string(),
                        BuildProvider::TeamCity,
                        self.info.id.clone(),
                        found.project_id.clone(),
                        found.project_name.clone(),
                        found.id.clone(),
                        found.name.clone(),
                        build.number.clone(),
                        build.get_build_status(),
                        if branch.name == "<default>" {
                            "default".to_string()
                        } else {
                            branch.name.clone()
                        },
                        build.url.clone(),
                        date::to_iso8601(&build.started_at, date::TEAMCITY_FORMAT)?,
                        match build.finished_at {
                            Option::None => None,
                            Option::Some(value) => {
                                Option::Some(date::to_iso8601(&value[..], date::TEAMCITY_FORMAT)?)
                            }
                        },
                    ));
                }
            }

            // Wait for a litle time between calls.
            if handle.wait(std::time::Duration::from_millis(300)).unwrap() {
                return Ok(());
            }
        }

        Ok(())
    }
}
