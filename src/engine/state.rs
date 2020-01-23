use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::builds::{Build, BuildStatus};
use crate::config::Configuration;
use crate::providers::collectors::CollectorInfo;

pub struct EngineState {
    pub title: String,
    pub builds: BuildRepository,
}

impl EngineState {
    pub fn new(config: &Configuration) -> Self {
        return EngineState {
            title: config.get_title().to_string(),
            builds: BuildRepository::new(),
        };
    }
}

pub struct BuildRepository {
    builds: Mutex<Vec<Build>>,
    statuses: Mutex<HashMap<u64, BuildStatus>>,
}

#[derive(PartialEq)]
pub enum BuildUpdateResult {
    Added,
    BuildUpdated,
    BuildStatusChanged,
    Unchanged,
}

impl BuildRepository {
    pub fn new() -> Self {
        Self {
            builds: Mutex::new(Vec::new()),
            statuses: Mutex::new(HashMap::new()),
        }
    }

    pub fn all(&self) -> Vec<Build> {
        self.builds.lock().unwrap().clone()
    }

    #[allow(clippy::block_in_if_condition_stmt)] // Clippy does not like what fmt does...
    pub fn update(&self, build: &Build) -> BuildUpdateResult {
        let mut builds = self.builds.lock().unwrap();

        // Is this exact build with the same status
        // already tracked? No need to update it then.
        if builds.iter().any(|b| {
            b.id == build.id && b.build_number == build.build_number && b.status == build.status
        }) {
            return BuildUpdateResult::Unchanged;
        }

        // Are we updating or adding the build?
        let mut result = if builds.iter().any(|b| b.partition == build.partition) {
            // This is an update to a build
            BuildUpdateResult::BuildUpdated
        } else {
            // This is a new build
            BuildUpdateResult::Added
        };

        // Did the absolute build status for the build change?
        // This either means success->failed or failed->success
        let mut statuses = self.statuses.lock().unwrap();
        if !statuses.contains_key(&build.partition) {
            statuses.insert(build.partition, build.status.clone());
        } else if let Some(val) = statuses.get_mut(&build.partition) {
            if build.status != BuildStatus::Running && *val != build.status {
                result = BuildUpdateResult::BuildStatusChanged;
                *val = build.status.clone();
            }
        }

        // Remove the build from the list
        builds.retain(|b| {
            !(b.collector == build.collector
                && b.project_id == build.project_id
                && b.definition_id == build.definition_id
                && b.build_id == build.build_id)
        });

        builds.push(build.clone());

        return result;
    }

    pub fn retain_builds(
        &self,
        collector_info: &CollectorInfo,
        build_ids: std::collections::HashSet<u64>,
    ) {
        // Remove all builds for the collector that was not
        // part of the provided list.
        let mut builds = self.builds.lock().unwrap();
        builds.retain(|b| {
            return !(b.provider == collector_info.provider
                && b.collector == collector_info.id
                && !build_ids.contains(&b.id));
        });

        // Only keep statuses that have corresponding builds.
        let mut statuses = self.statuses.lock().unwrap();
        statuses.retain(|id, _| builds.iter().any(|b| &b.partition == id));
    }

    pub fn current_status(&self) -> BuildStatus {
        let results = self.builds.lock().unwrap();
        if results.len() == 0 {
            return BuildStatus::Unknown;
        }
        if results.iter().any(|b| b.status == BuildStatus::Running) {
            return BuildStatus::Running;
        } else if results.iter().any(|b| b.status == BuildStatus::Failed) {
            return BuildStatus::Failed;
        }
        BuildStatus::Success
    }

    pub fn current_status_for_collectors(&self, collectors: &HashSet<String>) -> BuildStatus {
        let results = self.builds.lock().unwrap();
        if !results.iter().any(|b| collectors.contains(&b.collector)) {
            return BuildStatus::Unknown;
        }
        if results
            .iter()
            .any(|b| collectors.contains(&b.collector) && b.status == BuildStatus::Running)
        {
            return BuildStatus::Running;
        } else if results
            .iter()
            .any(|b| collectors.contains(&b.collector) && b.status == BuildStatus::Failed)
        {
            return BuildStatus::Failed;
        }
        BuildStatus::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::BuildProvider;

    fn create_build<T: Into<String>>(
        id: T,
        collector: T,
        project: T,
        definition: T,
        branch: T,
        status: BuildStatus,
    ) -> Build {
        Build::new(
            id.into(),
            BuildProvider::TeamCity,
            collector.into(),
            project.into(),
            "project".to_string(),
            definition.into(),
            "definition".to_string(),
            "1".to_string(),
            status,
            branch.into(),
            "https://dummy".to_string(),
            "".to_string(),
            Option::None,
        )
    }

    #[test]
    fn should_have_successful_as_current_state_if_there_are_no_builds() {
        let state = BuildRepository::new();
        assert!(state.current_status() == BuildStatus::Unknown);
    }

    #[test]
    fn should_set_state_to_running_if_one_build_is_running() {
        let state = BuildRepository::new();
        state.update(&create_build(
            "1",
            "collector",
            "project1",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project2",
            "ci/cd",
            "develop",
            BuildStatus::Running,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project3",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        assert!(state.current_status() == BuildStatus::Running);
    }

    #[test]
    fn should_set_state_to_failed_if_one_build_is_failed() {
        let state = BuildRepository::new();
        state.update(&create_build(
            "1",
            "collector",
            "project1",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project2",
            "ci/cd",
            "develop",
            BuildStatus::Failed,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project3",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        assert!(state.current_status() == BuildStatus::Failed);
    }

    #[test]
    fn should_set_state_to_running_even_if_there_are_failed_builds() {
        let state = BuildRepository::new();
        state.update(&create_build(
            "1",
            "collector",
            "project1",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project2",
            "ci/cd",
            "develop",
            BuildStatus::Running,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project3",
            "ci/cd",
            "develop",
            BuildStatus::Failed,
        ));
        state.update(&create_build(
            "1",
            "collector",
            "project4",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        assert!(state.current_status() == BuildStatus::Running);
    }

    #[test]
    fn should_return_correct_state_for_specific_collectors() {
        let state = BuildRepository::new();
        state.update(&create_build(
            "1",
            "collector1",
            "project1",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));
        state.update(&create_build(
            "1",
            "collector1",
            "project2",
            "ci/cd",
            "develop",
            BuildStatus::Running,
        ));
        state.update(&create_build(
            "1",
            "collector2",
            "project3",
            "ci/cd",
            "develop",
            BuildStatus::Failed,
        ));
        state.update(&create_build(
            "1",
            "collector1",
            "project4",
            "ci/cd",
            "develop",
            BuildStatus::Success,
        ));

        let mut collectors = HashSet::<String>::new();
        collectors.insert("collector2".to_string());

        assert!(state.current_status_for_collectors(&collectors) == BuildStatus::Failed);
    }
}
