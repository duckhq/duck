use log::{debug, trace};
use reqwest::header::ACCEPT;
use reqwest::{Client, ClientBuilder, RequestBuilder};
use url::Url;

use crate::builds::BuildStatus;
use crate::config::OctopusDeployCredentials;
use crate::utils::DuckResult;

pub struct OctopusDeployClient {
    url: Url,
    credentials: OctopusDeployCredentials,
    client: Client,
}

impl OctopusDeployCredentials {
    fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder {
        return match self {
            OctopusDeployCredentials::ApiKey(api_key) => {
                builder.header("X-Octopus-ApiKey", api_key)
            }
        };
    }
}

impl OctopusDeployClient {
    pub fn new(server_url: Url, credentials: OctopusDeployCredentials) -> Self {
        OctopusDeployClient {
            url: server_url,
            credentials,
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn get_dashboard(&self) -> DuckResult<OctopusDashboard> {
        let mut response =
            self.send_get_request(&format!("{url}api/dashboard", url = self.url,)[..])?;
        let result: OctopusDashboard = response.json()?;
        Ok(result)
    }

    fn send_get_request(&self, url: &str) -> DuckResult<reqwest::Response> {
        debug!("Sending request to: {}", url);
        let response = self.client.get(url).header(ACCEPT, "application/json");
        let response = self.credentials.authenticate(response).send()?;

        trace!("Received response: {}", response.status());
        if !response.status().is_success() {
            return Err(format_err!(
                "Received non 200 HTTP status code. {}",
                response.status()
            ));
        }

        Ok(response)
    }
}

#[derive(Deserialize, Debug)]
pub struct OctopusDashboard {
    #[serde(rename = "Projects")]
    pub projects: Vec<OctopusProject>,
    #[serde(rename = "Environments")]
    pub environments: Vec<OctopusEnvironment>,
    #[serde(rename = "Items")]
    pub deployments: Vec<OctopusDeployment>,
}

impl OctopusDashboard {
    pub fn find_project(&self, id: &str) -> Option<&OctopusProject> {
        self.projects.iter().find(|&p| p.id == id)
    }

    pub fn get_environment(&self, name: &str) -> Option<&OctopusEnvironment> {
        self.environments.iter().find(|&e| e.id == name)
    }

    pub fn find_deployment(
        &self,
        project: &OctopusProject,
        environment: &OctopusEnvironment,
    ) -> Option<&OctopusDeployment> {
        self.deployments
            .iter()
            .find(|&d| d.project == project.id && d.environment == environment.id)
    }
}

#[derive(Deserialize, Debug)]
pub struct OctopusProject {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Slug")]
    pub slug: String,
    #[serde(rename = "EnvironmentIds")]
    pub environments: Vec<String>,
}

impl OctopusProject {
    pub fn has_environment(&self, id: &str) -> bool {
        self.environments.iter().any(|e| e == id)
    }
}

#[derive(Deserialize, Debug)]
pub struct OctopusEnvironment {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct OctopusDeployment {
    #[serde(rename = "Id")]
    pub id: String,
    #[serde(rename = "ProjectId")]
    pub project: String,
    #[serde(rename = "EnvironmentId")]
    pub environment: String,
    #[serde(rename = "ReleaseId")]
    pub release_id: String,
    #[serde(rename = "ReleaseVersion")]
    pub release_version: String,
    #[serde(rename = "State")]
    pub status: String,
    #[serde(rename = "Links")]
    pub links: OctopusDeploymentLinks,
    #[serde(rename = "Created")]
    pub created_time: String,
    #[serde(rename = "QueueTime")]
    pub queue_time: Option<String>,
    #[serde(rename = "StartTime")]
    pub start_time: Option<String>,
    #[serde(rename = "CompletedTime")]
    pub finish_time: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OctopusDeploymentLinks {
    #[serde(rename = "Self")]
    pub deployment: String,
}

impl OctopusDeployment {
    pub fn get_status(&self) -> BuildStatus {
        match &self.status[..] {
            "Success" => BuildStatus::Success,
            "Executing" | "Queued" | "Cancelling" | "Canceled" | "Failed" | "" => {
                BuildStatus::Running
            }
            _ => BuildStatus::Failed,
        }
    }

    pub fn get_start_time(&self) -> String {
        match &self.status[..] {
            "Cancelling" | "Canceled" | "Success" | "Executing" => self.start_time.clone(),
            "Queued" => self.queue_time.clone(),
            _ => None,
        }
        .unwrap_or_else(|| self.created_time.clone())
    }
}
