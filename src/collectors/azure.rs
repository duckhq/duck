use std::sync::Arc;

use log::trace;
use reqwest::header::ACCEPT;
use reqwest::{Client, ClientBuilder, RequestBuilder};
use waithandle::{EventWaitHandle, WaitHandle};

use crate::builds::{Build, BuildProvider, BuildStatus};
use crate::collectors::{Collector, CollectorInfo};
use crate::config::{AzureDevOpsConfiguration, AzureDevOpsCredentials};
use crate::utils::date;
use crate::utils::DuckResult;

#[allow(dead_code)]
pub struct AzureDevOpsCollector {
    client: AzureDevOpsClient,
    branches: Vec<String>,
    definitions: Vec<String>,
    info: CollectorInfo,
}

impl AzureDevOpsCollector {
    pub fn new(config: &AzureDevOpsConfiguration) -> Self {
        return AzureDevOpsCollector {
            client: AzureDevOpsClient::new(config),
            branches: config.branches.clone(),
            definitions: config.definitions.clone(),
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
}

impl Collector for AzureDevOpsCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        handle: Arc<EventWaitHandle>,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        for branch in self.branches.iter() {
            if handle.check().unwrap() {
                return Ok(());
            }

            let builds = self.client.get_builds(branch, &self.definitions)?;
            for build in builds.value.iter() {
                callback(Build::new(
                    build.id.to_string(),
                    BuildProvider::AzureDevOps,
                    self.info.id.clone(),
                    build.project.id.clone(),
                    build.project.name.clone(),
                    build.definition.id.to_string(),
                    build.definition.name.clone(),
                    build.build_number.clone(),
                    build.get_build_status(),
                    build.branch.clone(),
                    build.links.web.href.clone(),
                    date::to_iso8601(&build.start_time, date::AZURE_DEVOPS_FORMAT)?,
                    match &build.finish_time {
                        Option::None => None,
                        Option::Some(value) => {
                            Option::Some(date::to_iso8601(&value[..], date::AZURE_DEVOPS_FORMAT)?)
                        }
                    },
                ));
            }

            // Wait for a litle time between calls.
            if handle.wait(std::time::Duration::from_millis(300)).unwrap() {
                return Ok(());
            }
        }

        return Ok(());
    }
}

#[derive(Deserialize, Debug)]
struct AzureResponse {
    pub value: Vec<AzureBuild>,
}

#[derive(Deserialize, Debug)]
struct AzureBuild {
    pub id: u64,
    #[serde(alias = "buildNumber")]
    pub build_number: String,
    pub project: AzureProject,
    pub definition: AzureBuildDefinition,
    pub status: String,
    pub result: Option<String>,
    #[serde(alias = "startTime")]
    pub start_time: String,
    #[serde(alias = "finishTime")]
    pub finish_time: Option<String>,
    #[serde(alias = "sourceBranch")]
    pub branch: String,
    #[serde(alias = "_links")]
    pub links: AzureLinks,
}

#[derive(Deserialize, Debug)]
struct AzureLinks {
    pub web: AzureWebLink,
}

#[derive(Deserialize, Debug)]
struct AzureWebLink {
    pub href: String,
}

impl AzureBuild {
    fn get_build_status(&self) -> BuildStatus {
        if self.result.is_none() {
            return BuildStatus::Running;
        } else {
            if self.status == "inProgress" || self.status == "notStarted" {
                return BuildStatus::Running;
            }
            match self.result.as_ref().unwrap().as_ref() {
                "succeeded" => BuildStatus::Success,
                _ => BuildStatus::Failed,
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct AzureProject {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct AzureBuildDefinition {
    pub id: u64,
    pub name: String,
}

struct AzureDevOpsClient {
    pub organization: String,
    pub project: String,
    client: Client,
    credentials: AzureDevOpsCredentials,
}

impl AzureDevOpsCredentials {
    fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder {
        match self {
            AzureDevOpsCredentials::Anonymous => builder,
            AzureDevOpsCredentials::PersonalAccessToken(token) => {
                (builder.basic_auth("", Some(token)))
            }
        }
    }
}

impl AzureDevOpsClient {
    pub fn new(config: &AzureDevOpsConfiguration) -> Self {
        AzureDevOpsClient {
            organization: config.organization.clone(),
            project: config.project.clone(),
            client: ClientBuilder::new().build().unwrap(),
            credentials: config.credentials.clone(),
        }
    }

    pub fn get_builds(&self, branch: &str, definitions: &[String]) -> DuckResult<AzureResponse> {
        // Get all branches for this build configuration.
        let mut response = self.send_get_request(format!(
            "https://dev.azure.com/{organization}/{project}/_apis/build/builds?api-version=5.1\
             &branchName={branch}&definitions={definitions}&maxBuildsPerDefinition=1\
             &queryOrder=startTimeDescending&deletedFilter=excludeDeleted\
             &statusFilter=cancelling,completed,inProgress",
            organization = self.organization,
            project = self.project,
            branch = branch,
            definitions = definitions.join(","),
        ))?;

        let result: AzureResponse = response.json()?;
        Ok(result)
    }

    fn send_get_request(&self, url: String) -> DuckResult<reqwest::Response> {
        trace!("Sending request to: {}", url);
        let response = self.client.get(&url).header(ACCEPT, "application/json");
        let response = self.credentials.authenticate(response).send()?;

        trace!("Received response: {}", response.status());
        if !response.status().is_success() {
            return Err(format_err!(
                "Received non 200 HTTP status code. ({})",
                response.status()
            ));
        }

        Ok(response)
    }
}
