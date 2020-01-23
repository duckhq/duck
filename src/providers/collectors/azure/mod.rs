use std::sync::Arc;

use waithandle::{EventWaitHandle, WaitHandle};

use crate::builds::{Build, BuildProvider, BuildStatus};
use crate::config::AzureDevOpsConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo};
use crate::utils::{date, DuckResult};

use self::client::*;

mod client;
mod validation;

#[allow(dead_code)]
pub struct AzureDevOpsCollector {
    client: AzureDevOpsClient,
    branches: Vec<String>,
    definitions: Vec<String>,
    info: CollectorInfo,
}

impl AzureDevOpsCollector {
    pub fn new(config: &AzureDevOpsConfiguration) -> Self {
        return AzureDevOpsCollector {
            client: AzureDevOpsClient::new(config),
            branches: config.branches.clone(),
            definitions: config.definitions.clone(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: BuildProvider::AzureDevOps,
            },
        };
    }
}

impl Collector for AzureDevOpsCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        handle: Arc<EventWaitHandle>,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        for branch in self.branches.iter() {
            if handle.check().unwrap() {
                return Ok(());
            }

            let builds = self.client.get_builds(branch, &self.definitions)?;
            for build in builds.value.iter() {
                callback(Build::new(
                    build.id.to_string(),
                    BuildProvider::AzureDevOps,
                    self.info.id.clone(),
                    build.project.id.clone(),
                    build.project.name.clone(),
                    build.definition.id.to_string(),
                    build.definition.name.clone(),
                    build.build_number.clone(),
                    build.get_build_status(),
                    build.branch.clone(),
                    build.links.web.href.clone(),
                    date::to_iso8601(&build.start_time, date::AZURE_DEVOPS_FORMAT)?,
                    match &build.finish_time {
                        Option::None => None,
                        Option::Some(value) => {
                            Option::Some(date::to_iso8601(&value[..], date::AZURE_DEVOPS_FORMAT)?)
                        }
                    },
                ));
            }

            // Wait for a litle time between calls.
            if handle.wait(std::time::Duration::from_millis(300)).unwrap() {
                return Ok(());
            }
        }

        return Ok(());
    }
}

impl AzureBuild {
    pub fn get_build_status(&self) -> BuildStatus {
        if self.result.is_none() {
            return BuildStatus::Running;
        } else {
            if self.status == "inProgress" || self.status == "notStarted" {
                return BuildStatus::Running;
            }
            match self.result.as_ref().unwrap().as_ref() {
                "succeeded" => BuildStatus::Success,
                "canceled" => BuildStatus::Canceled,
                _ => BuildStatus::Failed,
            }
        }
    }
}
