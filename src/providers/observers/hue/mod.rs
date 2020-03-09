use std::collections::HashSet;
use std::iter::FromIterator;

use log::debug;

use crate::config::HueConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo, ObserverLoader};
use crate::utils::http::{HttpClient, ReqwestClient};
use crate::DuckResult;

use self::client::HueClient;

mod client;
mod validation;

impl ObserverLoader for HueConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Observer>> {
        Ok(Box::new(HueObserver::<ReqwestClient> {
            client: HueClient::new(self),
            http: Default::default(),
            info: ObserverInfo {
                id: self.id.clone(),
                enabled: match self.enabled {
                    None => true,
                    Some(e) => e,
                },
                collectors: match &self.collectors {
                    Option::None => Option::None,
                    Option::Some(collectors) => {
                        Some(HashSet::from_iter(collectors.iter().cloned()))
                    }
                },
            },
        }))
    }
}

pub struct HueObserver<T: HttpClient + Default> {
    client: HueClient,
    http: T,
    info: ObserverInfo,
}

impl<T: HttpClient + Default> HueObserver<T> {
    #[cfg(test)]
    pub fn new(config: &HueConfiguration) -> Self {
        HueObserver {
            client: HueClient::new(config),
            http: Default::default(),
            info: ObserverInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    None => true,
                    Some(e) => e,
                },
                collectors: match &config.collectors {
                    Option::None => Option::None,
                    Option::Some(collectors) => {
                        Some(HashSet::from_iter(collectors.iter().cloned()))
                    }
                },
            },
        }
    }

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Observer for HueObserver<T> {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        match observation {
            Observation::DuckStatusChanged(status) => {
                debug!(
                    "[{}] Setting light state to '{:?}'...",
                    self.info.id, status
                );
                self.client.set_state(&self.http, status)?;
            }
            Observation::ShuttingDown => {
                debug!("[{}] Turning off all lights...", self.info.id);
                self.client.turn_off(&self.http)?;
            }
            _ => {}
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
    use test_case::test_case;

    #[test]
    fn should_post_to_correct_url() {
        // Given
        let hue = HueObserver::<MockHttpClient>::new(&HueConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            brightness: Some(255),
            collectors: None,
            hub_url: "https://example.com".to_string(),
            username: "patrik".to_string(),
            lights: vec!["foo".to_string()],
        });

        let client = hue.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Put,
                "https://example.com/api/patrik/lights/foo/state",
            )
            .returns_status(StatusCode::OK),
        );

        // When
        hue.observe(Observation::DuckStatusChanged(BuildStatus::Success))
            .unwrap();

        // Then
        let requests = client.get_sent_requests();
        assert_eq!(1, requests.len());
        assert_eq!(HttpMethod::Put, requests[0].method);
        assert_eq!(
            "https://example.com/api/patrik/lights/foo/state",
            &requests[0].url
        );
    }

    #[test_case(BuildStatus::Success, "{\"alert\":\"none\",\"xy\":[0.32114217,0.59787315],\"on\":true,\"bri\":255}" ; "Success")]
    #[test_case(BuildStatus::Failed, "{\"alert\":\"select\",\"xy\":[0.64842725,0.3308561],\"on\":true,\"bri\":255}" ; "Failed")]
    #[test_case(BuildStatus::Running, "{\"alert\":\"none\",\"xy\":[0.29151475,0.33772817],\"on\":true,\"bri\":255}" ; "Running")]
    fn should_send_correct_payload(status: BuildStatus, expected: &str) {
        // Given
        let hue = HueObserver::<MockHttpClient>::new(&HueConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            brightness: Some(255),
            collectors: None,
            hub_url: "https://example.com".to_string(),
            username: "patrik".to_string(),
            lights: vec!["foo".to_string()],
        });

        let client = hue.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Put,
                "https://example.com/api/patrik/lights/foo/state",
            )
            .returns_status(StatusCode::OK),
        );

        // When
        hue.observe(Observation::DuckStatusChanged(status)).unwrap();

        // Then
        let requests = client.get_sent_requests();
        assert_eq!(1, requests.len());
        assert!(&requests[0].body.is_some());
        assert_eq!(expected, &requests[0].body.clone().unwrap());
    }

    #[test]
    #[should_panic(expected = "Could not update state for light \\'foo\\' (502 Bad Gateway)")]
    fn should_return_error_if_server_return_non_successful_http_status_code() {
        // Given
        let hue = HueObserver::<MockHttpClient>::new(&HueConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            brightness: Some(255),
            collectors: None,
            hub_url: "https://example.com".to_string(),
            username: "patrik".to_string(),
            lights: vec!["foo".to_string()],
        });

        let client = hue.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(
                HttpMethod::Put,
                "https://example.com/api/patrik/lights/foo/state",
            )
            .returns_status(StatusCode::BAD_GATEWAY),
        );

        // When, Then
        hue.observe(Observation::DuckStatusChanged(BuildStatus::Success))
            .unwrap();
    }
}
