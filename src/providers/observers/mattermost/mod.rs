use std::collections::HashSet;
use std::iter::FromIterator;

use log::info;

use crate::builds::BuildStatus;
use crate::config::MattermostConfiguration;
use crate::filters::BuildFilter;
use crate::providers::observers::{Observation, Observer, ObserverInfo, ObserverLoader};
use crate::utils::http::{HttpClient, ReqwestClient};
use crate::DuckResult;

use self::client::MattermostClient;

mod client;
mod validation;

impl ObserverLoader for MattermostConfiguration {
    fn load(&self) -> DuckResult<Box<dyn Observer>> {
        Ok(Box::new(MattermostObserver::<ReqwestClient>::new(self)?))
    }
}

pub struct MattermostObserver<T: HttpClient + Default> {
    client: MattermostClient,
    http: T,
    info: ObserverInfo,
}

impl<T: HttpClient + Default> MattermostObserver<T> {
    pub fn new(config: &MattermostConfiguration) -> DuckResult<Self> {
        Ok(MattermostObserver {
            client: MattermostClient::new(config),
            http: Default::default(),
            info: ObserverInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    None => true,
                    Some(e) => e,
                },
                filter: BuildFilter::new(config.filter.clone())?,
                collectors: match &config.collectors {
                    Option::None => Option::None,
                    Option::Some(collectors) => {
                        Some(HashSet::from_iter(collectors.iter().cloned()))
                    }
                },
            },
        })
    }

    #[cfg(test)]
    pub fn get_client(&self) -> &T {
        &self.http
    }
}

impl<T: HttpClient + Default> Observer for MattermostObserver<T> {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        if let Observation::BuildStatusChanged(build) = observation {
            if build.status != BuildStatus::Unknown {
                info!(
                    "Sending Mattermost message since build status changed ({})...",
                    build.status
                );
                self.client.send(
                    &self.http,
                    &format!(
                        "{} build status for {}::{} ({}) changed to *{:?}*",
                        build.provider,
                        build.project_name,
                        build.definition_name,
                        build.branch,
                        build.status
                    )[..],
                )?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{BuildBuilder, BuildStatus};
    use crate::config::MattermostCredentials;
    use crate::utils::http::{HttpMethod, MockHttpClient, MockHttpResponseBuilder};
    use reqwest::StatusCode;
    use test_case::test_case;

    #[test]
    fn should_post_to_webhook_url() {
        // Given
        let mattermost = MattermostObserver::<MockHttpClient>::new(&MattermostConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            collectors: None,
            channel: None,
            filter: None,
            credentials: MattermostCredentials::Webhook {
                url: "https://example.com/webhook".to_string(),
            },
        })
        .unwrap();

        let client = mattermost.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Put, "https://example.com/webhook")
                .returns_status(StatusCode::OK),
        );

        // When
        mattermost
            .observe(Observation::BuildStatusChanged(
                &BuildBuilder::dummy().unwrap(),
            ))
            .unwrap();

        // Then
        let requests = client.get_sent_requests();
        assert_eq!(1, requests.len());
        assert_eq!(HttpMethod::Post, requests[0].method);
        assert_eq!("https://example.com/webhook", &requests[0].url);
    }

    #[test_case(BuildStatus::Success, "{\"text\":\"TeamCity build status for project_name::definition_name (branch) changed to *Success*\"}" ; "Success")]
    #[test_case(BuildStatus::Failed, "{\"text\":\"TeamCity build status for project_name::definition_name (branch) changed to *Failed*\"}" ; "Failed")]
    fn should_send_correct_payload(status: BuildStatus, expected: &str) {
        // Given
        let mattermost = MattermostObserver::<MockHttpClient>::new(&MattermostConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            collectors: None,
            channel: None,
            filter: None,
            credentials: MattermostCredentials::Webhook {
                url: "https://example.com/webhook".to_string(),
            },
        })
        .unwrap();

        let client = mattermost.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Put, "https://example.com/webhook")
                .returns_status(StatusCode::OK),
        );

        // When
        mattermost
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
    fn should_include_channel_in_payload_if_specified() {
        // Given
        let mattermost = MattermostObserver::<MockHttpClient>::new(&MattermostConfiguration {
            id: "hue".to_string(),
            enabled: Some(true),
            collectors: None,
            filter: None,
            channel: Some("foo".to_string()),
            credentials: MattermostCredentials::Webhook {
                url: "https://example.com/webhook".to_string(),
            },
        })
        .unwrap();

        let client = mattermost.get_client();
        client.add_response(
            MockHttpResponseBuilder::new(HttpMethod::Put, "https://example.com/webhook")
                .returns_status(StatusCode::OK),
        );

        // When
        mattermost
            .observe(Observation::BuildStatusChanged(
                &BuildBuilder::dummy().status(BuildStatus::Success).unwrap(),
            ))
            .unwrap();

        // Then
        let requests = client.get_sent_requests();
        assert_eq!(1, requests.len());
        assert!(&requests[0].body.is_some());
        assert_eq!(
            "{\"channel_id\":\"foo\",\"text\":\"TeamCity build status for project_name::definition_name (branch) changed to *Success*\"}",
            &requests[0].body.clone().unwrap()
        );
    }
}
