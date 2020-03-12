use log::{trace, warn};

use crate::builds::BuildStatus;
use crate::config::{AppVeyorCredentials, AppVeyorConfiguration};
use crate::utils::date;
use crate::utils::http::*;
use crate::DuckResult;

pub struct AppVeyorClient {
    credentials: AppVeyorCredentials,
}

impl AppVeyorClient {
    pub fn new(config: &AppVeyorConfiguration) -> Self {
        Self {
            credentials: config.credentials.clone(),
        }
    }

    pub fn get_builds(
        &self,
        client: &impl HttpClient,
        account: &str,
        project: &str,
        count: u16,
    ) -> DuckResult<AppVeyorResponse> {
        let url = format!(
            "https://ci.appveyor.com/api/projects/{account}/{project}/history?recordsNumber={count}",
            account = account,
            project = project,
            count = count
        );

        trace!("Sending request to: {}", url);
        let mut builder = HttpRequestBuilder::get(&url);
        builder.add_header("Content-Type", "application/json");
        builder.add_header("Accept", "application/json");

        self.credentials.authenticate(&mut builder);
        let mut response = client.send(&builder)?;

        trace!("Received response: {}", response.status());
        if !response.status().is_success() {
            return Err(format_err!(
                "Received non 200 HTTP status code. ({})",
                response.status()
            ));
        }

        // Get the response body.
        let body = response.body()?;
        // Deserialize and return the value.
        Ok(serde_json::from_str(&body[..])?)
    }
}

impl AppVeyorCredentials {
    fn authenticate<'a>(&self, builder: &'a mut HttpRequestBuilder) {
        match self {
            AppVeyorCredentials::Bearer(token) => {
                builder.bearer(token);
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct AppVeyorResponse {
    pub project: AppVeyorProject,
    pub builds: Vec<AppVeyorBuild>,
}

#[derive(Deserialize, Debug)]
pub struct AppVeyorProject {
    #[serde(alias = "accountId")]
    pub account_id: u64,
    #[serde(alias = "accountName")]
    pub account_name: String,
    #[serde(alias = "projectId")]
    pub project_id: u64,
    #[serde(alias = "name")]
    pub project_name: String,
    #[serde(alias = "repositoryName")]
    pub repository_name: String,
}

#[derive(Deserialize, Debug)]
pub struct AppVeyorBuild {
    #[serde(alias = "buildId")]
    pub build_id: u64,
    #[serde(alias = "buildNumber")]
    pub build_number: u64,
    pub branch: String,
    pub status: String,
    pub created: String,
    pub started: Option<String>,
    pub finished: Option<String>,
}

impl AppVeyorBuild {
    pub fn get_status(&self) -> BuildStatus {
        match &self.status[..] {
            "success" => BuildStatus::Success,
            "queued" => BuildStatus::Queued,
            "starting" => BuildStatus::Queued,
            "running" => BuildStatus::Running,
            "failed" => BuildStatus::Failed,
            "cancelled" => BuildStatus::Canceled,
            status => {
                warn!("Unknown build status: {}", status);
                BuildStatus::Unknown
            }
        }
    }

    pub fn get_started_timestamp(&self) -> DuckResult<i64> {
        let started = match &self.started {
            Some(started) => started,
            None => &self.created,
        };
        date::to_timestamp(&started[..], date::APPVEYOR_FORMAT)
    }

    pub fn get_finished_timestamp(&self) -> DuckResult<Option<i64>> {
        if let Some(finished) = &self.finished {
            let ts = date::to_timestamp(&finished[..], date::APPVEYOR_FORMAT)?;
            return Ok(Some(ts));
        }
        Ok(None)
    }
}
