use std::sync::Arc;

use log::warn;
use waithandle::{EventWaitHandle, WaitHandle};

use jenkins_api::{JenkinsBuilder, Jenkins};
use jenkins_api::build::{BuildStatus as JobBuildStatus, CommonBuild};

use crate::builds::{Build, BuildBuilder, BuildProvider, BuildStatus};
use crate::config::{JenkinsConfiguration, JenkinsCredentials};
use crate::providers::collectors::{Collector, CollectorInfo};
use crate::utils::{DuckResult};

mod validation;

pub struct JenkinsCollector {
    client: Jenkins,
    jobs: Vec<String>,
    info: CollectorInfo
}

impl JenkinsCollector {
    pub fn new(config: &JenkinsConfiguration) -> Self {
        let JenkinsCredentials::Basic { username, password } = &config.credentials;
        let jenkins = JenkinsBuilder::new(&config.server_url)
            .with_user(&username, Some(&password))
            .build().expect("Failed to create a jenkins client");

        return Self {
            client: jenkins,
            jobs: config.jobs.clone(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(enabled) => enabled,
                },
                provider: BuildProvider::Jenkins
            }
        }
    }
}

fn get_build_status(build: &CommonBuild) -> BuildStatus {
    if build.building {
        return BuildStatus::Running;
    }

    match build.result {
        Some(status) => {
            match status {
                JobBuildStatus::Success => BuildStatus::Success,
                JobBuildStatus::Aborted => BuildStatus::Canceled,
                JobBuildStatus::Failure => BuildStatus::Failed,
                JobBuildStatus::NotBuilt => BuildStatus::Queued,
                _ => BuildStatus::Unknown,
            }
        },
        None => BuildStatus::Unknown
    }
}

impl Collector for JenkinsCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(&self, handle: Arc<EventWaitHandle>, callback: &mut dyn FnMut(Build)) -> DuckResult<()> {

        for job_name in self.jobs.iter() {
            let job = match self.client.get_job(job_name) {
                Ok(j) => j,
                Err(_) => {
                    warn!("Job '{}' couldn't be requested.", job_name);
                    continue;
                }
            };

            let build = match job.last_build {
                Some(b) => b,
                None => {
                    warn!("The Job '{}' hasn't been built yet.", job_name);
                    continue;
                }
            };

            let build = match build.get_full_build(&self.client) {
                Ok(b) => b,
                Err(_) => {
                    warn!("Failed to be full build info for build {} of '{}'", build.number, job_name);
                    continue;
                }
            };

            callback(
                BuildBuilder::new()
                    .build_id(&build.id)
                    .provider(BuildProvider::Jenkins)
                    .collector(&self.info.id)
                    .definition_id(job_name)
                    .definition_name(&job.display_name)
                    .build_number(format!("{}", &build.number))
                    .status(get_build_status(&build))
                    .url(&build.url)
                    .build()
                    .unwrap(),
            );

            // Wait for a little time between calls.
            if handle.wait(std::time::Duration::from_millis(300)).unwrap() {
                return Ok(());
            }

        }

        Ok(())
    }
}