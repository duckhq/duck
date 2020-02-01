use std::collections::HashSet;
use std::iter::FromIterator;

use log::info;

use crate::builds::BuildStatus;
use crate::config::SlackConfiguration;
use crate::providers::observers::{Observation, Observer, ObserverInfo};
use crate::utils::http::HttpClient;
use crate::utils::DuckResult;

use self::client::SlackClient;

mod client;
mod validation;

pub struct SlackObserver<T: HttpClient + Default> {
    client: SlackClient,
    http: T,
    info: ObserverInfo,
}

impl<T: HttpClient + Default> SlackObserver<T> {
    pub fn new(config: &SlackConfiguration) -> Self {
        SlackObserver {
            client: SlackClient::new(config),
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

impl<T: HttpClient + Default> Observer for SlackObserver<T> {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        if let Observation::BuildStatusChanged(build) = observation {
            if is_interesting_status(&build.status) {
                info!(
                    "Sending Slack message since build status changed ({:?})...",
                    build.status
                );
                self.client.send(
                    &self.http,
                    &format!(
                        "{:?} build status for {}::{} ({}) changed to *{:?}*",
                        build.provider,
                        build.project_name,
                        build.definition_name,
                        build.branch,
                        build.status
                    )[..],
                    match build.status {
                        BuildStatus::Success => ":heavy_check_mark:",
                        BuildStatus::Failed => ":heavy_multiplication_x:",
                        _ => ":question:",
                    },
                )?;
            }
        };

        Ok(())
    }
}

fn is_interesting_status(status: &BuildStatus) -> bool {
    match status {
        BuildStatus::Success | BuildStatus::Failed => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{BuildBuilder, BuildStatus};
    use crate::config::SlackCredentials;
    use crate::utils::http::{HttpMethod, MockHttpClient, MockHttpResponseBuilder};
    use reqwest::StatusCode;
    use test_case::test_case;

    #[test]
    fn should_post_to_webhook_url() {
        // Given
        let slack = SlackObserver::<MockHttpClient>::new(&SlackConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            collectors: None,
            channel: None,
            credentials: SlackCredentials::Webhook {
                url: "https://example.com/webhook".to_string(),
            },
        });

        let client = slack.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Put, "https://example.com/webhook")
                .returns_status(StatusCode::OK),
        );

        // When
        slack
            .observe(Observation::BuildStatusChanged(
                &BuildBuilder::dummy().unwrap(),
            ))
            .unwrap();

        // Then
        let requests = client.get_sent_requests();
        assert_eq!(1, requests.len());
        assert_eq!(HttpMethod::Put, requests[0].method);
        assert_eq!("https://example.com/webhook", &requests[0].url);
    }

    #[test_case(BuildStatus::Success, "{\"icon_emoji\":\":heavy_check_mark:\",\"text\":\"TeamCity build status for project_name::definition_name (branch) changed to *Success*\",\"username\":\"Duck\"}" ; "Success")]
    #[test_case(BuildStatus::Failed, "{\"icon_emoji\":\":heavy_multiplication_x:\",\"text\":\"TeamCity build status for project_name::definition_name (branch) changed to *Failed*\",\"username\":\"Duck\"}" ; "Failed")]
    fn should_send_correct_payload(status: BuildStatus, expected: &str) {
        // Given
        let slack = SlackObserver::<MockHttpClient>::new(&SlackConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            collectors: None,
            channel: None,
            credentials: SlackCredentials::Webhook {
                url: "https://example.com/webhook".to_string(),
            },
        });

        let client = slack.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Put, "https://example.com/webhook")
                .returns_status(StatusCode::OK),
        );

        // When
        slack
            .observe(Observation::BuildStatusChanged(
                &BuildBuilder::dummy().status(status).unwrap(),
            ))
            .unwrap();

        // Then
        let requests = client.get_sent_requests();
        assert_eq!(1, requests.len());
        assert!(&requests[0].body.is_some());
        assert_eq!(expected, &requests[0].body.clone().unwrap());
    }

    #[test]
    #[should_panic(expected = "Could not send Slack message (502 Bad Gateway)")]
    fn should_return_error_if_server_return_non_successful_http_status_code() {
        // Given
        let slack = SlackObserver::<MockHttpClient>::new(&SlackConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            collectors: None,
            channel: None,
            credentials: SlackCredentials::Webhook {
                url: "https://example.com/webhook".to_string(),
            },
        });

        let client = slack.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Put, "https://example.com/webhook")
                .returns_status(StatusCode::BAD_GATEWAY),
        );

        // When
        slack
            .observe(Observation::BuildStatusChanged(
                &BuildBuilder::dummy().unwrap(),
            ))
            .unwrap();

        // When, Then
        slack
            .observe(Observation::DuckStatusChanged(BuildStatus::Success))
            .unwrap();
    }
}
