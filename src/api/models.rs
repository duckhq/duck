use serde::Serialize;

use crate::builds::{Build, BuildStatus};

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
    pub started: String,
    pub finished: Option<String>,
    pub url: String,
    pub status: BuildStatusViewModel,
}

#[derive(Serialize, Clone)]
pub enum BuildStatusViewModel {
    Unknown,
    Success,
    Failed,
    Running,
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
            started: item.started_at.clone(),
            finished: item.finished_at.clone(),
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
        }
    }
}
