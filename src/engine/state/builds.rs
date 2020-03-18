use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;

use crate::builds::{Build, BuildStatus};
use crate::engine::state::views::ViewRepository;
use crate::providers::collectors::CollectorInfo;

pub struct BuildRepository {
    builds: Mutex<Vec<Build>>,
    statuses: Mutex<HashMap<u64, BuildStatus>>,
}

#[derive(PartialEq)]
pub enum BuildUpdateResult {
    Added,
    BuildUpdated,
    AbsoluteBuildStatusChanged,
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

    pub fn for_view(&self, views: &ViewRepository, id: &str) -> Vec<Build> {
        let builds = self.builds.lock().unwrap();

        if let Some(collectors) = views.get_collectors(id) {
            let mut result = Vec::<Build>::new();
            for build in builds.iter() {
                if collectors.contains(&build.collector) {
                    result.push(build.clone());
                }
            }
            return result;
        }

        return vec![];
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
            if build.status.is_absolute() && *val != build.status {
                result = BuildUpdateResult::AbsoluteBuildStatusChanged;
                *val = build.status.clone();
            }
        }

        // Remove the build from the list and add it again.
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

    /// Retains all builds that belong to the provided collectors.
    pub fn retain(&self, collectors: &HashSet<String>) {
        let mut builds = self.builds.lock().unwrap();
        builds.retain(|b| {
            return collectors.contains(&b.collector[..]);
        });
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
    use crate::builds::BuildBuilder;

    #[test]
    fn should_have_successful_as_current_state_if_there_are_no_builds() {
        let state = BuildRepository::new();
        assert!(state.current_status() == BuildStatus::Unknown);
    }

    #[test]
    fn should_set_state_to_running_if_one_build_is_running() {
        let state = BuildRepository::new();
        state.update(
            &BuildBuilder::dummy()
                .build_id("1")
                .collector("collector")
                .project_id("project1")
                .definition_id("ci/cd")
                .branch("develop")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .build_id("1")
                .collector("collector")
                .project_id("project2")
                .definition_id("ci/cd")
                .branch("develop")
                .status(BuildStatus::Running)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .build_id("1")
                .collector("collector")
                .project_id("project3")
                .definition_id("ci/cd")
                .branch("develop")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        assert!(state.current_status() == BuildStatus::Running);
    }

    #[test]
    fn should_set_state_to_failed_if_one_build_is_failed() {
        let state = BuildRepository::new();
        state.update(
            &BuildBuilder::dummy()
                .project_id("project1")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .project_id("project2")
                .status(BuildStatus::Failed)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .project_id("project3")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        assert!(state.current_status() == BuildStatus::Failed);
    }

    #[test]
    fn should_set_state_to_running_even_if_there_are_failed_builds() {
        let state = BuildRepository::new();
        state.update(
            &BuildBuilder::dummy()
                .project_id("project1")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .project_id("project2")
                .status(BuildStatus::Running)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .project_id("project3")
                .status(BuildStatus::Failed)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .project_id("project4")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        assert!(state.current_status() == BuildStatus::Running);
    }

    #[test]
    fn should_return_correct_state_for_specific_collectors() {
        let state = BuildRepository::new();
        state.update(
            &BuildBuilder::dummy()
                .collector("collector1")
                .project_id("project1")
                .status(BuildStatus::Success)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .collector("collector1")
                .project_id("project2")
                .status(BuildStatus::Running)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .collector("collector2")
                .project_id("project3")
                .status(BuildStatus::Failed)
                .unwrap(),
        );
        state.update(
            &BuildBuilder::dummy()
                .collector("collector1")
                .project_id("project4")
                .status(BuildStatus::Success)
                .unwrap(),
        );

        let mut collectors = HashSet::<String>::new();
        collectors.insert("collector2".to_string());

        assert!(state.current_status_for_collectors(&collectors) == BuildStatus::Failed);
    }
}
