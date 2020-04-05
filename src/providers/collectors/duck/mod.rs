use log::info;
use waithandle::WaitHandleListener;

use crate::builds::{Build, BuildBuilder};
use crate::config::DuckConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo, CollectorLoader};
use crate::utils::http::{HttpClient, ReqwestClient};
use crate::utils::switch::Switch;
use crate::DuckResult;

use self::client::DuckClient;

mod client;
mod validation;

impl CollectorLoader for DuckConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(DuckCollector::<ReqwestClient>::new(self)))
    }
}

pub struct DuckCollector<T: HttpClient + Default> {
    http: T,
    client: DuckClient,
    server_url: String,
    version_error_switch: Switch,
    info: CollectorInfo,
}

impl<T: HttpClient + Default> DuckCollector<T> {
    pub fn new(config: &DuckConfiguration) -> Self {
        return DuckCollector {
            http: Default::default(),
            client: DuckClient::new(config),
            server_url: config.server_url.clone(),
            version_error_switch: Switch::new(false),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: "Duck".to_owned(),
            },
        };
    }

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Collector for DuckCollector<T> {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        _handle: WaitHandleListener,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        // Make sure we're on the same version as the server
        let version = self.client.get_server_version(&self.http)?;
        if version != crate::utils::VERSION {
            if self.version_error_switch.is_off() {
                self.version_error_switch.turn_on();
                return Err(format_err!(
                    "The remote duck server version is {} ({} is required)",
                    version,
                    crate::utils::VERSION
                ));
            }
            return Ok(());
        } else {
            if self.version_error_switch.is_on() {
                info!("Now fetching builds from remote Duck server")
            }
            self.version_error_switch.turn_off();
        }

        let builds = self.client.get_builds(&self.http)?;
        for build in builds {
            callback(
                BuildBuilder::new()
                    .build_id(&build.build_id)
                    .provider(&build.provider)
                    .origin(&self.server_url)
                    .collector(&self.info.id)
                    .project_id(&build.project)
                    .project_name(&build.project)
                    .definition_id(&build.build)
                    .definition_name(&build.build)
                    .build_number(&build.build_number)
                    .status(build.get_status())
                    .url(&build.url)
                    .started_at(build.started)
                    .finished_at(build.finished)
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

    fn create_collector(view: Option<String>) -> DuckCollector<MockHttpClient> {
        DuckCollector::<MockHttpClient>::new(&DuckConfiguration {
            id: "duck_other".to_owned(),
            enabled: Some(true),
            server_url: "http://localhost:15826".to_owned(),
            view,
        })
    }

    #[test]
    fn should_return_correct_provider_name() {
        // Given
        let github = create_collector(None);
        // When
        let provider = &github.info().provider;
        // Then
        assert_eq!("Duck", provider);
    }

    #[test]
    fn should_get_correct_data() {
        // Given
        let duck = create_collector(None);
        let client = duck.get_client();

        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Get, "http://localhost:15826/api/server")
                .returns_status(StatusCode::OK)
                .returns_body(format!("{{ \"version\": \"{}\" }}", crate::utils::VERSION)),
        );
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Get, "http://localhost:15826/api/builds")
                .returns_status(StatusCode::OK)
                .returns_body(include_str!("test_data/builds.json")),
        );

        let (_, listener) = waithandle::new();

        // When
        let mut result = Vec::<Build>::new();
        duck.collect(listener, &mut |build: Build| {
            // Store the results
            result.push(build);
        })
        .unwrap();

        // Then
        assert_eq!(8, result.len());
        assert_eq!("9767", result[0].build_id);
        assert_eq!("AzureDevOps", result[0].provider);
        assert_eq!("duck_other", result[0].collector);
        assert_eq!("Cake", result[0].project_id);
        assert_eq!("Cake", result[0].project_name);
        assert_eq!(
            "Azure Pipelines - Build Cake Centos 7",
            result[0].definition_id
        );
        assert_eq!(
            "Azure Pipelines - Build Cake Centos 7",
            result[0].definition_name
        );
        assert_eq!("9767", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("refs/heads/develop", result[0].branch);
        assert_eq!(
            "https://dev.azure.com/cake-build/af63183c-ac1f-4dbb-93bc-4fa862ea5809/_build/results?buildId=9767",
            result[0].url
        );
        assert_eq!(1584846026, result[0].started_at);
        assert_eq!(1584846262, result[0].finished_at.unwrap());
    }

    #[test]
    fn should_get_correct_data_for_view() {
        // Given
        let duck = create_collector(Some("foo".to_owned()));
        let client = duck.get_client();

        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Get, "http://localhost:15826/api/server")
                .returns_status(StatusCode::OK)
                .returns_body(format!("{{ \"version\": \"{}\" }}", crate::utils::VERSION)),
        );
        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Get,
                "http://localhost:15826/api/builds/view/foo",
            )
            .returns_status(StatusCode::OK)
            .returns_body(include_str!("test_data/view.json")),
        );

        let (_, listener) = waithandle::new();

        // When
        let mut result = Vec::<Build>::new();
        duck.collect(listener, &mut |build: Build| {
            // Store the results
            result.push(build);
        })
        .unwrap();

        // Then
        assert_eq!(1, result.len());
        assert_eq!("58880314", result[0].build_id);
        assert_eq!("GitHub", result[0].provider);
        assert_eq!("duck_other", result[0].collector);
        assert_eq!("spectresystems/duck", result[0].project_id);
        assert_eq!("spectresystems/duck", result[0].project_name);
        assert_eq!("ci.yaml", result[0].definition_id);
        assert_eq!("ci.yaml", result[0].definition_name);
        assert_eq!("24", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("setup-docker-for-local-development", result[0].branch);
        assert_eq!(
            "https://github.com/spectresystems/duck/actions/runs/58880314",
            result[0].url
        );
        assert_eq!(1584617069, result[0].started_at);
        assert_eq!(1584617318, result[0].finished_at.unwrap());
    }
}
