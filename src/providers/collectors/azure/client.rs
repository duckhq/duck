use log::trace;
use url::Url;

use crate::config::{AzureDevOpsConfiguration, AzureDevOpsCredentials};
use crate::utils::http::*;
use crate::DuckResult;

pub struct AzureDevOpsClient {
    server_url: Url,
    organization: String,
    project: String,
    credentials: AzureDevOpsCredentials,
}

impl AzureDevOpsClient {
    pub fn new(config: &AzureDevOpsConfiguration) -> Self {
        AzureDevOpsClient {
            server_url: match &config.server_url {
                Some(url) => Url::parse(&url[..]).unwrap(),
                None => Url::parse("https://dev.azure.com").unwrap(),
            },
            organization: config.organization.clone(),
            project: config.project.clone(),
            credentials: config.credentials.clone(),
        }
    }

    pub fn get_origin(&self) -> String {
        format!(
            "{}{}/{}",
            self.server_url.as_str(),
            self.organization,
            self.project
        )
    }

    pub fn get_builds(
        &self,
        client: &impl HttpClient,
        branch: &str,
        definitions: &[String],
    ) -> DuckResult<AzureResponse> {
        let url = format!(
            "{server}{organization}/{project}/_apis/build/builds?api-version=5.0\
             &branchName={branch}&definitions={definitions}&maxBuildsPerDefinition=1\
             &queryOrder=startTimeDescending&deletedFilter=excludeDeleted\
             &statusFilter=cancelling,completed,inProgress",
            server = self.server_url,
            organization = self.organization,
            project = self.project,
            branch = branch,
            definitions = definitions.join(","),
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

impl AzureDevOpsCredentials {
    fn authenticate(&self, builder: &mut HttpRequestBuilder) {
        match self {
            AzureDevOpsCredentials::Anonymous => {}
            AzureDevOpsCredentials::PersonalAccessToken(token) => {
                builder.basic_auth("", Some(token))
            }
        }
    }
}

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
