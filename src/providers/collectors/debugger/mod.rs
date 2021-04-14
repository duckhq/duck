use waithandle::WaitHandleListener;

use crate::builds::{Build, BuildBuilder};
use crate::config::DebuggerConfiguration;
use crate::providers::collectors::{Collector, CollectorInfo, CollectorLoader};
use crate::utils::date;
use crate::utils::http::{HttpClient, ReqwestClient};
use crate::DuckResult;

mod client;
mod validation;

use self::client::DebuggerClient;

impl CollectorLoader for DebuggerConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Collector>> {
        Ok(Box::new(DebuggerCollector::<ReqwestClient>::new(self)))
    }
}

pub struct DebuggerCollector<T: HttpClient + Default> {
    http: T,
    client: DebuggerClient,
    server_url: String,
    info: CollectorInfo,
}

impl<T: HttpClient + Default> DebuggerCollector<T> {
    pub fn new(config: &DebuggerConfiguration) -> Self {
        return DebuggerCollector {
            http: Default::default(),
            client: DebuggerClient::new(config),
            server_url: config.server_url.clone(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: config.enabled.unwrap_or(true),
                provider: "Debugger".to_owned(),
            },
        };
    }

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Collector for DebuggerCollector<T> {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        _handle: WaitHandleListener,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        let builds = self.client.get_builds(&self.http)?;
        for build in builds {
            callback(
                BuildBuilder::new()
                    .build_id(&build.id.to_string())
                    .provider("Debugger")
                    .origin(&self.server_url)
                    .collector(&self.info.id)
                    .project_id(&build.project)
                    .project_name(&build.project)
                    .definition_id(&build.definition)
                    .definition_name(&build.definition)
                    .build_number(&build.id.to_string())
                    .branch(&build.branch)
                    .status(build.get_status())
                    .url(format!("{}/Edit?id={}", &self.server_url, build.id))
                    .started_at(date::to_timestamp(&build.started, date::DEBUGGER_FORMAT)?)
                    .finished_at(match &build.finished {
                        Option::None => None,
                        Option::Some(value) => {
                            Option::Some(date::to_timestamp(&value[..], date::DEBUGGER_FORMAT)?)
                        }
                    })
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
    use crate::utils::http::{HttpMethod, MockHttpClient, MockHttpResponseBuilder};
    use reqwest::StatusCode;

    fn create_collector() -> DebuggerCollector<MockHttpClient> {
        DebuggerCollector::<MockHttpClient>::new(&DebuggerConfiguration {
            id: "debug".to_owned(),
            enabled: Some(true),
            server_url: "http://localhost:5000".to_owned(),
        })
    }

    #[test]
    fn should_return_correct_provider_name() {
        // Given
        let debugger = create_collector();
        // When
        let provider = &debugger.info().provider;
        // Then
        assert_eq!("Debugger", provider);
    }

    #[test]
    fn should_get_correct_data() {
        // Given
        let debugger = create_collector();
        let (_, listener) = waithandle::new();
        let client = debugger.get_client();

        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Get, "http://localhost:5000/api/builds")
                .returns_status(StatusCode::OK)
                .returns_body(include_str!("test_data/builds.json")),
        );

        // When
        let mut result = Vec::<Build>::new();
        debugger
            .collect(listener, &mut |build: Build| {
                // Store the results
                result.push(build);
            })
            .unwrap();

        // Then
        assert_eq!(2, result.len());
        assert_eq!("1", result[0].build_id);
        assert_eq!("Debugger", result[0].provider);
        assert_eq!("debug", result[0].collector);
        assert_eq!("Cauldron", result[0].project_id);
        assert_eq!("Cauldron", result[0].project_name);
        assert_eq!("Debug", result[0].definition_id);
        assert_eq!("Debug", result[0].definition_name);
        assert_eq!("1", result[0].build_number);
        assert_eq!(BuildStatus::Success, result[0].status);
        assert_eq!("master", result[0].branch);
        assert_eq!("http://localhost:5000/Edit?id=1", result[0].url);
        assert_eq!(1586811562, result[0].started_at);
        assert_eq!(1586811827, result[0].finished_at.unwrap());
    }
}
