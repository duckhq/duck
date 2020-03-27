use log::{error, trace, warn};
use waithandle::WaitHandleListener;

use crate::builds::{Build, BuildBuilder, BuildStatus};
use crate::config::TeamCityConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo};
use crate::utils::date;
use crate::DuckResult;

use self::client::*;
use super::CollectorLoader;

mod client;
mod validation;

impl CollectorLoader for TeamCityConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(TeamCityCollector {
            client: TeamCityClient::new(self),
            build_types: self.builds.clone(),
            info: CollectorInfo {
                id: self.id.clone(),
                enabled: match self.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: "TeamCity".to_string(),
            },
        }))
    }
}

pub struct TeamCityCollector {
    client: TeamCityClient,
    build_types: Vec<String>,
    info: CollectorInfo,
}

impl Collector for TeamCityCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        listener: WaitHandleListener,
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
            if listener.check().unwrap() {
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
                if listener.check().unwrap() {
                    return Ok(());
                }

                let branch_name = if branch.name == "<default>" {
                    "default"
                } else {
                    &branch.name
                };

                match branch.builds.builds.first() {
                    None => trace!("No builds found for branch '{}'", branch_name),
                    Some(build) => {
                        callback(
                            BuildBuilder::new()
                                .build_id(build.id.to_string())
                                .provider("TeamCity")
                                .origin(self.client.url.as_str())
                                .collector(&self.info.id)
                                .project_id(&found.project_id)
                                .project_name(&found.project_name)
                                .definition_id(&found.id)
                                .definition_name(&found.name)
                                .build_number(&build.number)
                                .status(build.get_build_status())
                                .url(&build.url)
                                .started_at(date::to_timestamp(
                                    &build.started_at,
                                    date::TEAMCITY_FORMAT,
                                )?)
                                .finished_at(build.get_finished_at()?)
                                .branch(branch_name)
                                .build()
                                .unwrap(),
                        );
                    }
                };
            }

            // Wait for a little time between calls.
            if listener
                .wait(std::time::Duration::from_millis(300))
                .unwrap()
            {
                return Ok(());
            }
        }

        Ok(())
    }
}

impl TeamCityBuildModel {
    pub fn get_build_status(&self) -> BuildStatus {
        return if self.running {
            BuildStatus::Running
        } else {
            match self.status.as_ref() {
                "SUCCESS" => BuildStatus::Success,
                "UNKNOWN" => BuildStatus::Canceled,
                _ => BuildStatus::Failed,
            }
        };
    }
}
