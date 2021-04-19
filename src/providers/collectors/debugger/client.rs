use log::trace;

use crate::builds::BuildStatus;
use crate::config::DebuggerConfiguration;
use crate::utils::http::*;
use crate::DuckResult;

pub struct DebuggerClient {
    pub server_url: String,
}

impl DebuggerClient {
    pub fn new(config: &DebuggerConfiguration) -> Self {
        Self {
            server_url: config.server_url.clone(),
        }
    }

    pub fn get_builds(&self, client: &impl HttpClient) -> DuckResult<Vec<DebuggerBuild>> {
        let url = format!("{server}/api/builds", server = self.server_url);
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
pub struct DebuggerBuild {
    pub id: u64,
    pub status: i8,
    pub started: String,
    pub finished: Option<String>,
    pub project: String,
    pub definition: String,
    pub branch: String,
}

impl DebuggerBuild {
    pub fn get_status(&self) -> BuildStatus {
        match self.status {
            0 => BuildStatus::Success,
            1 => BuildStatus::Failed,
            2 => BuildStatus::Running,
            3 => BuildStatus::Canceled,
            4 => BuildStatus::Queued,
            5 => BuildStatus::Skipped,
            _ => BuildStatus::Unknown,
        }
    }
}
