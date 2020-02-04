use std::sync::Arc;

use waithandle::{EventWaitHandle, WaitHandle};

use crate::builds::{Build, BuildBuilder, BuildProvider, BuildStatus};
use crate::config::AzureDevOpsConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo};
use crate::utils::{date, DuckResult};

use self::client::*;

mod client;
mod validation;

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
                callback(
                    BuildBuilder::new()
                        .build_id(build.id.to_string())
                        .provider(BuildProvider::AzureDevOps)
                        .collector(&self.info.id)
                        .project_id(&build.project.id)
                        .project_name(&build.project.name)
                        .definition_id(build.definition.id.to_string())
                        .definition_name(&build.definition.name)
                        .build_number(&build.build_number)
                        .status(build.get_build_status())
                        .url(&build.links.web.href)
                        .started_at(date::to_timestamp(
                            &build.start_time,
                            date::AZURE_DEVOPS_FORMAT,
                        )?)
                        .finished_at(match &build.finish_time {
                            Option::None => None,
                            Option::Some(value) => Option::Some(date::to_timestamp(
                                &value[..],
                                date::AZURE_DEVOPS_FORMAT,
                            )?),
                        })
                        .branch(&build.branch)
                        .build()
                        .unwrap(),
                );
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
            match &self.status[..] {
                "notStarted" => return BuildStatus::Queued,
                "inProgress" => return BuildStatus::Running,
                _ => { },
            }
            match self.result.as_ref().unwrap().as_ref() {
                "succeeded" => BuildStatus::Success,
                "canceled" => BuildStatus::Canceled,
                _ => BuildStatus::Failed,
            }
        }
    }
}
