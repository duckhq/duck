use log::trace;
use reqwest::header::ACCEPT;
use reqwest::{Client, ClientBuilder, RequestBuilder};
use url::Url;

use crate::config::{TeamCityAuth, TeamCityConfiguration};
use crate::utils::date;
use crate::utils::DuckResult;

pub struct TeamCityClient {
    pub url: Url,
    credentials: TeamCityAuth,
    client: Client,
}

impl TeamCityClient {
    pub fn new(settings: &TeamCityConfiguration) -> Self {
        Self {
            url: Url::parse(&settings.server_url[..]).unwrap(),
            credentials: settings.credentials.clone(),
            client: ClientBuilder::new().build().unwrap(),
        }
    }

    pub fn is_online(&self) -> bool {
        self.send_get_request(format!(
            "{url}{authtype}/app/rest/server",
            url = self.url,
            authtype = self.credentials.get_auth_type()
        ))
        .is_ok()
    }

    pub fn get_build_types(&self) -> DuckResult<Vec<TeamCityBuildTypeModel>> {
        // Get all branches for this build configuration.
        let mut response = self.send_get_request(format!(
            "{url}{authtype}/app/rest/buildTypes",
            url = self.url,
            authtype = self.credentials.get_auth_type()
        ))?;

        let result: TeamCityBuildTypeCollectionModel = response.json()?;

        Ok(result.build_types)
    }

    pub fn get_builds(
        &self,
        build_type: &TeamCityBuildTypeModel,
    ) -> DuckResult<TeamCityBranchCollectionModel> {
        // Get all branches for this build configuration.
        let mut response = self.send_get_request(format!(
            "{url}{authtype}/app/rest/buildTypes/id:{id}/branches?locator=default:any\
             &fields=count,branch(name,default,active,builds(build(id,number,running,status,\
             branchName,webUrl,startDate,finishDate),count,$locator(running:any,canceled:any,count:1)))",
            url = self.url,
            authtype = self.credentials.get_auth_type(),
            id = build_type.id
        ))?;

        let result: TeamCityBranchCollectionModel = response.json()?;

        Ok(result)
    }

    fn send_get_request(&self, url: String) -> DuckResult<reqwest::Response> {
        trace!("Sending request to: {}", url);
        let response = self.client.get(&url).header(ACCEPT, "application/json");
        let response = self.credentials.authenticate(response).send()?;

        trace!("Received response: {}", response.status());
        if !response.status().is_success() {
            return Err(format_err!("Received non 200 HTTP status code."));
        }

        Ok(response)
    }
}

impl TeamCityAuth {
    pub fn get_auth_type(&self) -> String {
        return match self {
            TeamCityAuth::Guest => "guestAuth".to_string(),
            TeamCityAuth::BasicAuth { .. } => "httpAuth".to_string(),
        };
    }
    pub fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder {
        return match self {
            TeamCityAuth::Guest => builder,
            TeamCityAuth::BasicAuth { username, password } => {
                (builder.basic_auth(username, Some(password)))
            }
        };
    }
}

#[derive(Deserialize, Debug)]
pub struct TeamCityBuildTypeCollectionModel {
    pub count: u32,
    #[serde(alias = "buildType")]
    pub build_types: Vec<TeamCityBuildTypeModel>,
}

#[derive(Deserialize, Debug)]
pub struct TeamCityBuildTypeModel {
    pub id: String,
    pub name: String,
    #[serde(alias = "projectId")]
    pub project_id: String,
    #[serde(alias = "projectName")]
    pub project_name: String,
}

#[derive(Deserialize, Debug)]
pub struct TeamCityBranchCollectionModel {
    pub count: u32,
    #[serde(alias = "branch")]
    pub branches: Vec<TeamCityBranchModel>,
}

#[derive(Deserialize, Debug)]
pub struct TeamCityBranchModel {
    pub name: String,
    #[serde(default)]
    pub default: bool,
    pub active: bool,
    pub builds: TeamCityBuildCollectionModel,
}

#[derive(Deserialize, Debug)]
pub struct TeamCityBuildCollectionModel {
    count: u32,
    #[serde(alias = "build")]
    pub builds: Vec<TeamCityBuildModel>,
}

#[derive(Deserialize, Debug)]
pub struct TeamCityBuildModel {
    pub id: u32,
    pub number: String,
    pub running: bool,
    pub status: String,
    #[serde(alias = "webUrl")]
    pub url: String,
    #[serde(alias = "startDate")]
    pub started_at: String,
    #[serde(alias = "finishDate")]
    pub finished_at: Option<String>,
}

impl TeamCityBuildModel {
    pub fn get_finished_at(&self) -> DuckResult<Option<String>> {
        let finished_at = match &self.finished_at {
            Option::None => None,
            Option::Some(value) => {
                Option::Some(date::to_iso8601(&value[..], date::TEAMCITY_FORMAT)?)
            }
        };
        Ok(finished_at)
    }
}
