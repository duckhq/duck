use log::trace;

use crate::builds::BuildStatus;
use crate::config::{GitHubConfiguration, GitHubCredentials};
use crate::utils::date;
use crate::utils::http::*;
use crate::DuckResult;

pub struct GitHubClient {
    pub owner: String,
    pub repository: String,
    pub workflow: String,
    credentials: GitHubCredentials,
    etag: std::sync::Mutex<Option<String>>,
    cached: std::sync::Mutex<Option<String>>,
}

impl GitHubClient {
    pub fn new(config: &GitHubConfiguration) -> Self {
        Self {
            owner: config.owner.clone(),
            repository: config.repository.clone(),
            workflow: config.workflow.clone(),
            credentials: config.credentials.clone(),
            etag: std::sync::Mutex::new(None),
            cached: std::sync::Mutex::new(None),
        }
    }

    pub fn get_builds(&self, client: &impl HttpClient) -> DuckResult<GitHubResponse> {
        let url = format!(
            "https://api.github.com/repos/{owner}/{repo}/actions/workflows/{workflow}/runs?page=0&per_page=25",
            owner = self.owner,
            repo = self.repository,
            workflow = self.workflow
        );

        trace!("Sending request to: {}", url);
        let mut builder = HttpRequestBuilder::get(&url);
        builder.add_header("Content-Type", "application/json");
        builder.add_header("Accept", "application/json");

        // Do we have an etag?
        let mut etag = self.etag.lock().unwrap();
        if etag.is_some() {
            let etag_value = etag.as_ref().unwrap();
            trace!("Using etag {}", etag_value);
            builder.add_header("If-None-Match", etag_value);
        }

        self.credentials.authenticate(&mut builder);
        let mut response = client.send(&builder)?;

        // Got an etag?
        let new_etag = response.headers().get("ETag");
        if let Some(new_etag) = new_etag {
            let new_etag_value = new_etag.to_str()?;
            trace!("Got a new etag: {}", new_etag_value);
            *etag = Some(new_etag_value.to_owned());
        }

        trace!("Received response: {}", response.status());
        if !response.status().is_success() {
            // Not modified?
            if response.status() == reqwest::StatusCode::NOT_MODIFIED {
                let cached = self.cached.lock().unwrap();
                if cached.is_none() {
                    return Err(format_err!(
                        "Got a 304 not modified, but we didn't have a response cached."
                    ));
                }
                return Ok(serde_json::from_str(&cached.as_ref().unwrap()[..])?);
            }
            return Err(format_err!(
                "Received non 200 HTTP status code. ({})",
                response.status()
            ));
        }

        // Get the response body.
        let body = response.body()?;

        // Cache the response. We need this if we return a 304 Not Modified.
        trace!("Cached the response from GitHub.");
        let mut cached = self.cached.lock().unwrap();
        *cached = Some(body.clone());

        // Deserialize and return the value.
        Ok(serde_json::from_str(&body[..])?)
    }
}

impl GitHubCredentials {
    fn authenticate<'a>(&self, builder: &'a mut HttpRequestBuilder) {
        match self {
            GitHubCredentials::Basic { username, password } => {
                builder.basic_auth(username, Some(password));
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GitHubResponse {
    pub total_count: u16,
    pub workflow_runs: Vec<GitHubWorkflowRun>,
}

#[derive(Deserialize, Debug)]
pub struct GitHubWorkflowRun {
    pub id: u64,
    #[serde(alias = "head_branch")]
    pub branch: String,
    #[serde(alias = "run_number")]
    pub number: u64,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
}

impl GitHubWorkflowRun {
    pub fn get_status(&self) -> DuckResult<BuildStatus> {
        match &self.status[..] {
            "completed" => match &self.conclusion {
                None => Err(format_err!("Build is completed without conclusion.")),
                Some(conclusion) => match &conclusion[..] {
                    "success" => Ok(BuildStatus::Success),
                    "cancelled" => Ok(BuildStatus::Canceled),
                    "failure" => Ok(BuildStatus::Failed),
                    "skipped" => Ok(BuildStatus::Skipped),
                    _ => Ok(BuildStatus::Failed),
                },
            },
            "queued" => Ok(BuildStatus::Queued),
            "in_progress" => Ok(BuildStatus::Running),
            status => Err(format_err!("Unknown build status '{}'", status)),
        }
    }

    pub fn get_started_timestamp(&self) -> DuckResult<i64> {
        let result = date::to_timestamp(&self.created_at, date::GITHUB_FORMAT)?;
        Ok(result)
    }

    pub fn get_finished_timestamp(&self) -> DuckResult<Option<i64>> {
        if self.status == "completed" {
            let result = date::to_timestamp(&self.updated_at, date::GITHUB_FORMAT)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}
