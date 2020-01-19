use std::sync::Arc;

use log::{error, trace, warn};
use reqwest::header::ACCEPT;
use reqwest::{Client, ClientBuilder, RequestBuilder};
use url::Url;
use waithandle::{EventWaitHandle, WaitHandle};

use crate::builds::{Build, BuildProvider, BuildStatus};
use crate::collectors::{Collector, CollectorInfo};
use crate::config::{TeamCityAuth, TeamCityConfiguration};
use crate::utils::date;
use crate::utils::DuckResult;

pub struct TeamCityCollector {
    client: TeamCityClient,
    build_types: Vec<String>,
    info: CollectorInfo,
}

impl TeamCityCollector {
    pub fn new(config: &TeamCityConfiguration) -> Self {
        return Self {
            client: TeamCityClient::new(config),
            build_types: config.builds.clone(),
            info: CollectorInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    Option::None => true,
                    Option::Some(e) => e,
                },
                provider: BuildProvider::TeamCity,
            },
        };
    }
}

impl Collector for TeamCityCollector {
    fn info(&self) -> &CollectorInfo {
        &self.info
    }

    fn collect(
        &self,
        handle: Arc<EventWaitHandle>,
        callback: &mut dyn FnMut(Build),
    ) -> DuckResult<()> {
        // Make sure TeamCity is online.
        if !self.client.is_online() {
            error!("There was a problem contacting TeamCity.");
            return Err(format_err!("There was a problem contacting TeamCity."));
        }

        // Get all known build types from TeamCity.
        let known_build_types = self.client.get_build_types()?;

        // Get builds for all build types.
        for build_type in self.build_types.iter() {
            if handle.check().unwrap() {
                return Ok(());
            }

            // Make sure the build type is known.
            let found = match known_build_types.iter().find(|t| t.id.eq(build_type)) {
                Option::None => {
                    warn!(
                        "The build type '{}' does not exist in TeamCity.",
                        build_type
                    );
                    continue;
                }
                Option::Some(r) => r,
            };

            trace!("Getting builds for {}...", build_type);
            let result = self.client.get_builds(found)?;
            for branch in result.branches {
                for build in branch.builds.builds {
                    callback(Build::new(
                        build.id.to_string(),
                        BuildProvider::TeamCity,
                        self.info.id.clone(),
                        found.project_id.clone(),
                        found.project_name.clone(),
                        found.id.clone(),
                        found.name.clone(),
                        build.number.clone(),
                        build.get_build_status(),
                        if branch.name == "<default>" {
                            "default".to_string()
                        } else {
                            branch.name.clone()
                        },
                        build.url.clone(),
                        date::to_iso8601(&build.started_at, date::TEAMCITY_FORMAT)?,
                        match build.finished_at {
                            Option::None => None,
                            Option::Some(value) => {
                                Option::Some(date::to_iso8601(&value[..], date::TEAMCITY_FORMAT)?)
                            }
                        },
                    ));
                }
            }

            // Wait for a litle time between calls.
            if handle.wait(std::time::Duration::from_millis(300)).unwrap() {
                return Ok(());
            }
        }

        Ok(())
    }
}

impl TeamCityAuth {
    fn get_auth_type(&self) -> String {
        return match self {
            TeamCityAuth::Guest => "guestAuth".to_string(),
            TeamCityAuth::BasicAuth { .. } => "httpAuth".to_string(),
        };
    }
    fn authenticate(&self, builder: RequestBuilder) -> RequestBuilder {
        return match self {
            TeamCityAuth::Guest => builder,
            TeamCityAuth::BasicAuth { username, password } => {
                (builder.basic_auth(username, Some(password)))
            }
        };
    }
}

#[derive(Deserialize, Debug)]
struct TeamCityBuildTypeCollectionModel {
    pub count: u32,
    #[serde(alias = "buildType")]
    pub build_types: Vec<TeamCityBuildTypeModel>,
}

#[derive(Deserialize, Debug)]
struct TeamCityBuildTypeModel {
    pub id: String,
    pub name: String,
    #[serde(alias = "projectId")]
    pub project_id: String,
    #[serde(alias = "projectName")]
    pub project_name: String,
}

#[derive(Deserialize, Debug)]
struct TeamCityBranchCollectionModel {
    pub count: u32,
    #[serde(alias = "branch")]
    pub branches: Vec<TeamCityBranchModel>,
}

#[derive(Deserialize, Debug)]
struct TeamCityBranchModel {
    pub name: String,
    #[serde(default)]
    pub default: bool,
    pub active: bool,
    pub builds: TeamCityBuildCollectionModel,
}

#[derive(Deserialize, Debug)]
struct TeamCityBuildCollectionModel {
    count: u32,
    #[serde(alias = "build")]
    pub builds: Vec<TeamCityBuildModel>,
}

#[derive(Deserialize, Debug)]
struct TeamCityBuildModel {
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
    fn get_build_status(&self) -> BuildStatus {
        return if self.running {
            BuildStatus::Running
        } else {
            match self.status.as_ref() {
                "SUCCESS" => BuildStatus::Success,
                _ => BuildStatus::Failed,
            }
        };
    }
}

struct TeamCityClient {
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
             branchName,webUrl,startDate,finishDate),count,$locator(running:any,count:1)))",
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
