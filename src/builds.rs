use std::collections::hash_map::DefaultHasher;
use std::fmt::Result as FormatResult;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub id: u64,
    pub partition: u64,
    pub build_id: String,
    pub provider: BuildProvider,
    pub collector: String,
    pub project_id: String,
    pub project_name: String,
    pub definition_id: String,
    pub definition_name: String,
    pub build_number: String,
    pub status: BuildStatus,
    pub branch: String,
    pub url: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    _private: (),
}

impl Build {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        build_id: String,
        provider: BuildProvider,
        collector: String,
        project_id: String,
        project_name: String,
        definition_id: String,
        definition_name: String,
        build_number: String,
        status: BuildStatus,
        branch: String,
        url: String,
        started_at: String,
        finished_at: Option<String>,
    ) -> Self {
        // Generate a hash that represents the build.
        let mut hasher = DefaultHasher::new();
        provider.hash(&mut hasher);
        collector.hash(&mut hasher);
        project_id.hash(&mut hasher);
        definition_id.hash(&mut hasher);
        branch.hash(&mut hasher);
        build_id.hash(&mut hasher);
        let id = hasher.finish();

        // Generate a hash that represents the build
        // definition (partition) of the build, not the build itself.
        let mut hasher = DefaultHasher::new();
        provider.hash(&mut hasher);
        collector.hash(&mut hasher);
        project_id.hash(&mut hasher);
        definition_id.hash(&mut hasher);
        branch.hash(&mut hasher);
        let partition = hasher.finish();

        Build {
            id,
            partition,
            build_id,
            provider,
            collector,
            project_id,
            project_name,
            definition_id,
            definition_name,
            build_number,
            status,
            branch,
            url,
            started_at,
            finished_at,
            _private: (),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BuildProvider {
    TeamCity,
    AzureDevOps,
}

impl Display for BuildProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            BuildProvider::TeamCity => write!(f, "TeamCity"),
            BuildProvider::AzureDevOps => write!(f, "Azure DevOps"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildStatus {
    Unknown,
    Success,
    Failed,
    Running,
}
