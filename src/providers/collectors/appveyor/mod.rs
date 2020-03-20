use waithandle::WaitHandleListener;

use crate::builds::{Build, BuildBuilder, BuildProvider};
use crate::config::AppVeyorConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo, CollectorLoader};
use crate::utils::http::*;
use crate::DuckResult;

use self::client::*;

mod client;
mod validation;

impl CollectorLoader for AppVeyorConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(AppVeyorCollector::<ReqwestClient>::new(self)))
    }
}

pub struct AppVeyorCollector<T: HttpClient + Default> {
    info: CollectorInfo,
    http: T,
    client: AppVeyorClient,
    account: String,
    project: String,
    count: u16,
}

impl<T: HttpClient + Default> AppVeyorCollector<T> {
    pub fn new(config: &AppVeyorConfiguration) -> Self {
        AppVeyorCollector::<T> {
            http: Default::default(),
            client: AppVeyorClient::new(config),
            account: config.account.clone(),
            project: config.project.clone(),
            count: config.get_count(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: BuildProvider::AppVeyor,
            },
        }
    }

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Collector for AppVeyorCollector<T> {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        listener: WaitHandleListener,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        let result =
            self.client
                .get_builds(&self.http, &self.account, &self.project, self.count)?;

        for (count, build) in result.builds.iter().enumerate() {
            if listener.check().unwrap() {
                break;
            }
            if count >= self.count as usize {
                break;
            }

            callback(
                BuildBuilder::new()
                    .build_id(build.build_id.to_string())
                    .provider(BuildProvider::AppVeyor)
                    .origin(format!(
                        "https://ci.appveyor.com/project/{account}/{project}",
                        account = self.account,
                        project = self.project
                    ))
                    .collector(&self.info.id)
                    .project_id(&result.project.project_id.to_string())
                    .project_name(&result.project.repository_name)
                    .definition_id(&result.project.account_id.to_string())
                    .definition_name(&result.project.account_name)
                    .build_number(&build.build_number.to_string())
                    .status(build.get_status())
                    .url(format!(
                        "https://ci.appveyor.com/project/{account}/{project}/builds/{id}",
                        account = self.account,
                        project = self.project,
                        id = build.build_id
                    ))
                    .started_at(build.get_started_timestamp()?)
                    .finished_at(build.get_finished_timestamp()?)
                    .branch(&build.branch)
                    .build()
                    .unwrap(),
            );
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
        let appveyor = AppVeyorCollector::<MockHttpClient>::new(&AppVeyorConfiguration {
            id: "appveyor".to_owned(),
            enabled: Some(true),
            account: "patriksvensson".to_owned(),
            project: "spectre-commandline".to_owned(),
            credentials: AppVeyorCredentials::Bearer("SECRET".to_owned()),
            count: Option::None,
        });

        let client = appveyor.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Get,
                "https://ci.appveyor.com/api/projects/patriksvensson/spectre-commandline/history?recordsNumber=1"
            )
            .returns_status(StatusCode::OK)
            .returns_body(include_str!("test_data/builds.json"))
        );

        let (_, listener) = waithandle::new();

        // When
        let mut result = Vec::<Build>::new();
        appveyor
            .collect(listener, &mut |build: Build| {
                // Store the results
                result.push(build);
            })
            .unwrap();

        // Then
        assert_eq!(1, result.len());
        assert_eq!("31395671", result[0].build_id);
        assert_eq!(BuildProvider::AppVeyor, result[0].provider);
        assert_eq!("appveyor", result[0].collector);
        assert_eq!("408686", result[0].project_id);
        assert_eq!("spectresystems/spectre.cli", result[0].project_name);
        assert_eq!("12349", result[0].definition_id);
        assert_eq!("patriksvensson", result[0].definition_name);
        assert_eq!("202", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("master", result[0].branch);
        assert_eq!(
            "https://ci.appveyor.com/project/patriksvensson/spectre-commandline/builds/31395671",
            result[0].url
        );
        assert_eq!(1583929960, result[0].started_at);
        assert_eq!(1583930062, result[0].finished_at.unwrap());
    }
}
