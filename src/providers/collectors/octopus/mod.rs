use log::warn;

use crate::config::{OctopusDeployConfiguration, OctopusDeployProject};
use crate::providers::collectors::*;
use crate::utils::date;

use self::client::OctopusDeployClient;

mod client;
mod validation;

pub struct OctopusDeployCollector {
    server_url: String,
    projects: Vec<OctopusDeployProject>,
    client: OctopusDeployClient,
    info: CollectorInfo,
}

impl OctopusDeployCollector {
    pub fn new(config: &OctopusDeployConfiguration) -> Self {
        OctopusDeployCollector {
            server_url: config.server_url.clone(),
            projects: config.projects.clone(),
            client: OctopusDeployClient::new(config.server_url.clone(), config.credentials.clone()),
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

                // Generate a new build.
                callback(Build::new(
                    deployment.id.clone(),
                    BuildProvider::OctopusDeploy,
                    self.info().id.clone(),
                    found_project.id.clone(),
                    found_project.name.clone(),
                    found_environment.id.clone(),
                    found_environment.name.clone(),
                    deployment.release_version.clone(),
                    deployment.get_status(),
                    deployment.release_id.clone(),
                    format!(
                        "{}/app#/projects/{}/releases/{}/deployments/{}",
                        self.server_url, found_project.slug, deployment.release_id, deployment.id
                    ),
                    date::to_iso8601(
                        &deployment.get_start_time()[..],
                        date::OCTOPUS_DEPLOY_FORMAT,
                    )?,
                    match &deployment.finish_time {
                        Option::None => None,
                        Option::Some(value) => {
                            Option::Some(date::to_iso8601(&value[..], date::OCTOPUS_DEPLOY_FORMAT)?)
                        }
                    },
                ));
            }
        }

        Ok(())
    }
}
