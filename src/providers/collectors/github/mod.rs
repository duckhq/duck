use std::sync::Arc;

use waithandle::EventWaitHandle;

use crate::builds::{Build, BuildBuilder, BuildProvider};
use crate::config::GitHubConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo, CollectorLoader};
use crate::utils::http::{HttpClient, ReqwestClient};
use crate::DuckResult;

use self::client::GitHubClient;

mod client;
mod validation;

impl CollectorLoader for GitHubConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(GitHubCollector::<ReqwestClient> {
            client: GitHubClient::new(self),
            http: Default::default(),
            info: CollectorInfo {
                id: self.id.clone(),
                enabled: match self.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: BuildProvider::AzureDevOps,
            },
        }))
    }
}

pub struct GitHubCollector<T: HttpClient + Default> {
    client: GitHubClient,
    http: T,
    info: CollectorInfo,
}

impl<T: HttpClient + Default> GitHubCollector<T> {
    #[cfg(test)]
    pub fn new(config: &GitHubConfiguration) -> Self {
        return GitHubCollector {
            client: GitHubClient::new(config),
            http: Default::default(),
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

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Collector for GitHubCollector<T> {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        _handle: Arc<EventWaitHandle>,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        let response = self.client.get_builds(&self.http)?;

        // Convert the workflow run to a Duck build representation.
        let mut builds = Vec::<Build>::new();
        for run in response.workflow_runs.iter() {
            builds.push(
                BuildBuilder::new()
                    .build_id(run.id.to_string())
                    .provider(BuildProvider::GitHub)
                    .origin(format!(
                        "{}/{}/{}",
                        &self.client.owner, &self.client.repository, &self.client.workflow
                    ))
                    .collector(&self.info.id)
                    .project_id(format!(
                        "{}_{}",
                        &self.client.owner, &self.client.repository
                    ))
                    .project_name(format!(
                        "{}/{}",
                        &self.client.owner, &self.client.repository
                    ))
                    .definition_id(&self.client.workflow)
                    .definition_name(&self.client.workflow)
                    .build_number(run.number.to_string())
                    .status(run.get_status()?)
                    .url(&run.html_url)
                    .started_at(run.get_started_timestamp()?)
                    .finished_at(run.get_finished_timestamp()?)
                    .branch(&run.branch)
                    .build()
                    .unwrap(),
            );
        }

        // Sort the builds by date.
        builds.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        // Now only keep the latest ones of the
        let mut branches = std::collections::HashSet::<&String>::new();
        for build in builds.iter() {
            if !branches.contains(&build.branch) {
                branches.insert(&build.branch);
                callback(build.clone()); // Really want to get rid of this clone...
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::BuildStatus;
    use crate::config::*;
    use crate::utils::http::{HttpMethod, MockHttpClient, MockHttpResponseBuilder};
    use reqwest::StatusCode;

    #[test]
    fn should_get_correct_data() {
        // Given
        let github = GitHubCollector::<MockHttpClient>::new(&GitHubConfiguration {
            id: "github".to_owned(),
            enabled: Some(true),
            owner: "spectresystems".to_owned(),
            repository: "duck".to_owned(),
            workflow: "pull_request.yml".to_owned(),
            credentials: GitHubCredentials::Basic {
                username: "foo".to_owned(),
                password: "lol".to_owned(),
            },
        });

        let client = github.get_client();

        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Get,
                "https://api.github.com/repos/spectresystems/duck/actions/workflows/pull_request.yml/runs?page=0&per_page=25"
            )
            .returns_status(StatusCode::OK)
            .returns_body(include_str!("test_data/builds.json"))
        );

        // When
        let mut result = Vec::<Build>::new();
        github
            .collect(
                Arc::new(waithandle::EventWaitHandle::new()),
                &mut |build: Build| {
                    // Store the results
                    result.push(build);
                },
            )
            .unwrap();

        // Then
        assert_eq!(4, result.len());
        assert_eq!("33801182", result[0].build_id);
        assert_eq!(BuildProvider::GitHub, result[0].provider);
        assert_eq!("github", result[0].collector);
        assert_eq!("spectresystems_duck", result[0].project_id);
        assert_eq!("spectresystems/duck", result[0].project_name);
        assert_eq!("pull_request.yml", result[0].definition_id);
        assert_eq!("pull_request.yml", result[0].definition_name);
        assert_eq!("52", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("feature/GH-30", result[0].branch);
        assert_eq!(
            "https://github.com/spectresystems/duck/actions/runs/33801182",
            result[0].url
        );
        assert_eq!(1580601850, result[0].started_at);
        assert_eq!(1580602219, result[0].finished_at.unwrap());
    }
}
