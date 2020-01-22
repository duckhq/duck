use log::trace;
use reqwest::header::ACCEPT;
use reqwest::{Client, ClientBuilder, RequestBuilder};

use crate::builds::BuildStatus;
use crate::config::{AzureDevOpsConfiguration, AzureDevOpsCredentials};
use crate::utils::DuckResult;

#[derive(Deserialize, Debug)]
pub struct AzureResponse {
    pub value: Vec<AzureBuild>,
}

#[derive(Deserialize, Debug)]
pub struct AzureBuild {
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
pub struct AzureLinks {
    pub web: AzureWebLink,
}

#[derive(Deserialize, Debug)]
pub struct AzureWebLink {
    pub href: String,
}

impl AzureBuild {
    pub fn get_build_status(&self) -> BuildStatus {
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
pub struct AzureProject {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct AzureBuildDefinition {
    pub id: u64,
    pub name: String,
}

pub struct AzureDevOpsClient {
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
