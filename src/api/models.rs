use serde::Serialize;

use crate::builds::{Build, BuildStatus};
use crate::config::ViewConfiguration;

#[derive(Serialize, Clone)]
pub struct ServerInfoModel<'a> {
    pub title: &'a str,
    pub version: &'static str,
    pub views: Vec<ViewInfoModel>,
}

#[derive(Serialize, Clone)]
pub struct ViewInfoModel {
    pub slug: String,
    pub name: String,
}

impl<'a> From<&ViewConfiguration> for ViewInfoModel {
    fn from(view: &ViewConfiguration) -> Self {
        ViewInfoModel {
            slug: view.id.clone(),
            name: view.name.clone(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct BuildViewModel {
    pub id: u64,
    pub provider: String,
    pub collector: String,
    pub project: String,
    pub build: String,
    pub branch: String,
    #[serde(rename(serialize = "buildId"))]
    pub build_id: String,
    #[serde(rename(serialize = "buildNumber"))]
    pub build_number: String,
    pub started: i64,
    pub finished: Option<i64>,
    pub url: String,
    pub status: BuildStatusViewModel,
}

#[derive(Serialize, Clone)]
pub enum BuildStatusViewModel {
    Unknown,
    Success,
    Failed,
    Running,
    Canceled,
    Queued,
}

impl From<&Build> for BuildViewModel {
    fn from(item: &Build) -> Self {
        BuildViewModel {
            id: item.id,
            provider: format!("{:?}", item.provider),
            collector: item.collector.clone(),
            project: item.project_name.clone(),
            build: item.definition_name.clone(),
            branch: item.branch.clone(),
            build_id: item.build_id.clone(),
            build_number: item.build_number.clone(),
            url: item.url.clone(),
            started: item.started_at,
            finished: item.finished_at,
            status: BuildStatusViewModel::from(&item.status),
        }
    }
}

impl From<&BuildStatus> for BuildStatusViewModel {
    fn from(item: &BuildStatus) -> Self {
        match item {
            BuildStatus::Unknown => BuildStatusViewModel::Unknown,
            BuildStatus::Success => BuildStatusViewModel::Success,
            BuildStatus::Failed => BuildStatusViewModel::Failed,
            BuildStatus::Running => BuildStatusViewModel::Running,
            BuildStatus::Canceled => BuildStatusViewModel::Canceled,
            BuildStatus::Queued => BuildStatusViewModel::Queued,
        }
    }
}
