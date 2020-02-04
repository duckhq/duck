use log::warn;
use url::Url;

use crate::builds::{BuildBuilder, BuildStatus};
use crate::config::{OctopusDeployConfiguration, OctopusDeployProject};
use crate::providers::collectors::*;
use crate::utils::date;

use self::client::*;

mod client;
mod validation;

pub struct OctopusDeployCollector {
    server_url: Url,
    projects: Vec<OctopusDeployProject>,
    client: OctopusDeployClient,
    info: CollectorInfo,
}

impl OctopusDeployCollector {
    pub fn new(config: &OctopusDeployConfiguration) -> Self {
        OctopusDeployCollector {
            server_url: Url::parse(&config.server_url[..]).unwrap(),
            projects: config.projects.clone(),
            client: OctopusDeployClient::new(
                Url::parse(&config.server_url[..]).unwrap(),
                config.credentials.clone(),
            ),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: BuildProvider::OctopusDeploy,
            },
        }
    }
}

impl Collector for OctopusDeployCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(&self, _: Arc<EventWaitHandle>, callback: &mut dyn FnMut(Build)) -> DuckResult<()> {
        let response = self.client.get_dashboard()?;

        for project in self.projects.iter() {
            // Get the project from the result.
            let found_project = match response.find_project(&project.project_id[..]) {
                Some(p) => p,
                None => {
                    warn!("Project '{}' does not exist.", project.project_id);
                    continue;
                }
            };

            for environment in project.environments.iter() {
                // Get the environment from the result.
                let found_environment = match response.get_environment(environment) {
                    Some(e) => e,
                    None => {
                        warn!("Environment '{}' does not exist.", environment);
                        continue;
                    }
                };

                // Does the project actually have the environment?
                if !found_project.has_environment(&found_environment.id[..]) {
                    warn!(
                        "Environment '{}' does not belong to project '{}'",
                        environment, found_project.name
                    );
                }

                // Get the deployment for the project and environment combination.
                let deployment = match response.find_deployment(found_project, found_environment) {
                    Some(d) => d,
                    None => {
                        warn!(
                            "No deployment found for Environment '{}' in project '{}'",
                            environment, found_project.name
                        );
                        continue;
                    }
                };

                callback(
                    BuildBuilder::new()
                        .build_id(&deployment.id)
                        .provider(BuildProvider::OctopusDeploy)
                        .collector(&self.info.id)
                        .project_id(&found_project.id)
                        .project_name(&found_project.name)
                        .definition_id(&found_environment.id)
                        .definition_name(&found_environment.name)
                        .build_number(&deployment.release_version)
                        .status(deployment.get_status())
                        .url(format!(
                            "{}/app#/projects/{}/releases/{}/deployments/{}",
                            self.server_url,
                            found_project.slug,
                            deployment.release_id,
                            deployment.id
                        ))
                        .started_at(date::to_timestamp(
                            &deployment.get_start_time()[..],
                            date::OCTOPUS_DEPLOY_FORMAT,
                        )?)
                        .finished_at(match &deployment.finish_time {
                            Option::None => None,
                            Option::Some(value) => Option::Some(date::to_timestamp(
                                &value[..],
                                date::OCTOPUS_DEPLOY_FORMAT,
                            )?),
                        })
                        .branch(&deployment.release_id)
                        .build()
                        .unwrap(),
                );
            }
        }

        Ok(())
    }
}

impl OctopusDashboard {
    pub fn find_project(&self, id: &str) -> Option<&OctopusProject> {
        self.projects.iter().find(|&p| p.id == id)
    }

    pub fn get_environment(&self, name: &str) -> Option<&OctopusEnvironment> {
        self.environments.iter().find(|&e| e.id == name)
    }

    pub fn find_deployment(
        &self,
        project: &OctopusProject,
        environment: &OctopusEnvironment,
    ) -> Option<&OctopusDeployment> {
        self.deployments
            .iter()
            .find(|&d| d.project == project.id && d.environment == environment.id)
    }
}

impl OctopusProject {
    pub fn has_environment(&self, id: &str) -> bool {
        self.environments.iter().any(|e| e == id)
    }
}

impl OctopusDeployment {
    pub fn get_status(&self) -> BuildStatus {
        match &self.status[..] {
            "Success" => BuildStatus::Success,
            "Queued" => BuildStatus::Queued,
            "Executing" | "Cancelling" | "Failed" | "" => BuildStatus::Running,
            "Canceled" => BuildStatus::Canceled,
            _ => BuildStatus::Failed,
        }
    }

    pub fn get_start_time(&self) -> String {
        match &self.status[..] {
            "Cancelling" | "Canceled" | "Success" | "Executing" => self.start_time.clone(),
            "Queued" => self.queue_time.clone(),
            _ => None,
        }
        .unwrap_or_else(|| self.created_time.clone())
    }
}
