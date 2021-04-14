use std::time::Duration;

use waithandle::WaitHandleListener;

use crate::builds::{Build, BuildBuilder, BuildStatus};
use crate::config::AzureDevOpsConfiguration;
use crate::providers::collectors::azure::client::{AzureBuild, AzureDevOpsClient};
use crate::providers::collectors::{Collector, CollectorInfo, CollectorLoader};
use crate::utils::date;
use crate::utils::http::*;
use crate::DuckResult;

mod client;
mod validation;

impl CollectorLoader for AzureDevOpsConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(AzureDevOpsCollector::<ReqwestClient>::new(self)))
    }
}

pub struct AzureDevOpsCollector<T: HttpClient + Default> {
    http: T,
    client: AzureDevOpsClient,
    branches: Vec<String>,
    definitions: Vec<String>,
    info: CollectorInfo,
}

impl<T: HttpClient + Default> AzureDevOpsCollector<T> {
    pub fn new(config: &AzureDevOpsConfiguration) -> Self {
        AzureDevOpsCollector {
            http: Default::default(),
            client: AzureDevOpsClient::new(config),
            branches: config.branches.clone(),
            definitions: config.definitions.clone(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: config.enabled.unwrap_or(true),
                provider: "AzureDevOps".to_string(),
            },
        }
    }

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Collector for AzureDevOpsCollector<T> {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        listener: WaitHandleListener,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        for branch in self.branches.iter() {
            if listener.check() {
                return Ok(());
            }

            let builds = self
                .client
                .get_builds(&self.http, branch, &self.definitions)?;
            for build in builds.value.iter() {
                if listener.check() {
                    return Ok(());
                }

                callback(
                    BuildBuilder::new()
                        .build_id(build.id.to_string())
                        .provider("AzureDevOps")
                        .origin(&self.client.get_origin())
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
            if listener.wait(Duration::from_millis(300)) {
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
                _ => {}
            }
            match self.result.as_ref().unwrap().as_ref() {
                "succeeded" => BuildStatus::Success,
                "canceled" => BuildStatus::Canceled,
                _ => BuildStatus::Failed,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::BuildStatus;
    use crate::config::*;
    use crate::utils::http::{HttpMethod, MockHttpClient, MockHttpResponseBuilder};
    use reqwest::StatusCode;

    fn create_collector(server_url: Option<String>) -> AzureDevOpsCollector<MockHttpClient> {
        AzureDevOpsCollector::<MockHttpClient>::new(&AzureDevOpsConfiguration {
            id: "azure".to_owned(),
            enabled: Some(true),
            server_url,
            organization: "cake-build".to_owned(),
            project: "cake".to_owned(),
            credentials: AzureDevOpsCredentials::PersonalAccessToken("SECRET".to_owned()),
            branches: vec!["refs/heads/develop".to_owned()],
            definitions: vec!["5".to_owned(), "6".to_owned()],
        })
    }

    #[test]
    fn should_return_correct_provider_name() {
        // Given
        let collector = create_collector(None);
        // When
        let provider = &collector.info().provider;
        // Then
        assert_eq!("AzureDevOps", provider);
    }

    #[test]
    fn should_get_correct_data_for_default_server_address() {
        // Given
        let collector = create_collector(None);
        let client = collector.get_client();

        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Get,
                "https://dev.azure.com/cake-build/cake/_apis/build/builds?api-version=5.0&branchName=refs/heads/develop&definitions=5,6&maxBuildsPerDefinition=1&queryOrder=startTimeDescending&deletedFilter=excludeDeleted&statusFilter=cancelling,completed,inProgress"
            )
            .returns_status(StatusCode::OK)
            .returns_body(include_str!("test_data/builds.json"))
        );

        let (_, listener) = waithandle::new();

        // When
        let mut result = Vec::<Build>::new();
        collector
            .collect(listener, &mut |build: Build| {
                // Store the results
                result.push(build);
            })
            .unwrap();

        // Then
        assert_eq!(2, result.len());
        assert_eq!("10059", result[0].build_id);
        assert_eq!("AzureDevOps", result[0].provider);
        assert_eq!("azure", result[0].collector);
        assert_eq!("https://dev.azure.com/cake-build/cake", result[0].origin);
        assert_eq!("af63183c-ac1f-4dbb-93bc-4fa862ea5809", result[0].project_id);
        assert_eq!("Cake", result[0].project_name);
        assert_eq!("5", result[0].definition_id);
        assert_eq!(
            "Azure Pipelines - Build Cake Centos 7",
            result[0].definition_name
        );
        assert_eq!("10059", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("refs/heads/develop", result[0].branch);
        assert_eq!(
            "https://dev.azure.com/cake-build/af63183c-ac1f-4dbb-93bc-4fa862ea5809/_build/results?buildId=10059",
            result[0].url
        );
        assert_eq!(1587697251, result[0].started_at);
        assert_eq!(1587697564, result[0].finished_at.unwrap());
    }

    #[test]
    fn should_get_correct_data_for_on_prem_server_address() {
        // Given
        let collector = create_collector(Some("https://foo.bar/".to_owned()));
        let client = collector.get_client();

        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Get,
                "https://foo.bar/cake-build/cake/_apis/build/builds?api-version=5.0&branchName=refs/heads/develop&definitions=5,6&maxBuildsPerDefinition=1&queryOrder=startTimeDescending&deletedFilter=excludeDeleted&statusFilter=cancelling,completed,inProgress"
            )
            .returns_status(StatusCode::OK)
            .returns_body(include_str!("test_data/builds.json"))
        );

        let (_, listener) = waithandle::new();

        // When
        let mut result = Vec::<Build>::new();
        collector
            .collect(listener, &mut |build: Build| {
                // Store the results
                result.push(build);
            })
            .unwrap();

        // Then
        assert_eq!(2, result.len());
        assert_eq!("10059", result[0].build_id);
        assert_eq!("AzureDevOps", result[0].provider);
        assert_eq!("azure", result[0].collector);
        assert_eq!("https://foo.bar/cake-build/cake", result[0].origin);
        assert_eq!("af63183c-ac1f-4dbb-93bc-4fa862ea5809", result[0].project_id);
        assert_eq!("Cake", result[0].project_name);
        assert_eq!("5", result[0].definition_id);
        assert_eq!(
            "Azure Pipelines - Build Cake Centos 7",
            result[0].definition_name
        );
        assert_eq!("10059", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("refs/heads/develop", result[0].branch);
        assert_eq!(
            "https://dev.azure.com/cake-build/af63183c-ac1f-4dbb-93bc-4fa862ea5809/_build/results?buildId=10059",
            result[0].url
        );
        assert_eq!(1587697251, result[0].started_at);
        assert_eq!(1587697564, result[0].finished_at.unwrap());
    }
}
