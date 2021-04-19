use log::trace;

use crate::builds::BuildStatus;
use crate::config::DuckConfiguration;
use crate::utils::http::*;
use crate::DuckResult;

pub struct DuckClient {
    pub server_url: String,
    pub view: Option<String>,
}

impl DuckClient {
    pub fn new(config: &DuckConfiguration) -> Self {
        Self {
            server_url: config.server_url.clone(),
            view: config.view.clone(),
        }
    }

    pub fn get_server_version(&self, client: &impl HttpClient) -> DuckResult<String> {
        let url = format!("{}/api/server", owner = self.server_url,);

        let body = self.send_get_request(client, url)?;
        let info: DuckServerInfo = serde_json::from_str(&body[..])?;

        Ok(info.version)
    }

    pub fn get_builds(&self, client: &impl HttpClient) -> DuckResult<Vec<DuckBuild>> {
        let url = match &self.view {
            Some(view) => format!(
                "{owner}/api/builds/view/{view}",
                owner = self.server_url,
                view = view
            ),
            None => format!("{owner}/api/builds", owner = self.server_url),
        };

        let body = self.send_get_request(client, url)?;
        Ok(serde_json::from_str(&body[..])?)
    }

    fn send_get_request(&self, client: &impl HttpClient, url: String) -> DuckResult<String> {
        trace!("Sending request to: {}", url);
        let mut builder = HttpRequestBuilder::get(&url);
        builder.add_header("Content-Type", "application/json");
        builder.add_header("Accept", "application/json");

        let mut response = client.send(&builder)?;

        trace!("Received response: {}", response.status());
        if !response.status().is_success() {
            return Err(format_err!(
                "Received non 200 HTTP status code. ({})",
                response.status()
            ));
        }

        response.body()
    }
}

#[derive(Deserialize, Debug)]
pub struct DuckServerInfo {
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct DuckBuild {
    pub id: u64,
    pub provider: String,
    pub collector: String,
    pub project: String,
    pub build: String,
    pub branch: String,
    #[serde(alias = "buildId")]
    pub build_id: String,
    #[serde(alias = "buildNumber")]
    pub build_number: String,
    pub started: i64,
    pub finished: Option<i64>,
    pub url: String,
    pub status: String,
}

impl DuckBuild {
    pub fn get_status(&self) -> BuildStatus {
        match &self.status[..] {
            "Success" => BuildStatus::Success,
            "Failed" => BuildStatus::Failed,
            "Running" => BuildStatus::Running,
            "Canceled" => BuildStatus::Canceled,
            "Queued" => BuildStatus::Queued,
            "Skipped" => BuildStatus::Skipped,
            _ => BuildStatus::Unknown,
        }
    }
}
