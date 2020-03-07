use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Builder, Debug, PartialEq, Eq)]
#[builder(field(private), build_fn(skip), setter(into), pattern = "immutable")] // TODO: Should not be immutable
pub struct Build {
    #[builder(setter(skip))]
    pub id: u64,
    #[builder(setter(skip))]
    pub partition: u64,
    pub origin: String,
    pub build_id: String,
    pub provider: String,
    pub collector: String,
    pub project_id: String,
    pub project_name: String,
    pub definition_id: String,
    pub definition_name: String,
    pub build_number: String,
    pub status: BuildStatus,
    pub branch: String,
    pub url: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

impl BuildBuilder {
    #[cfg(test)]
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        BuildBuilder::new()
            .build_id("foo")
            .provider("Dummy")
            .origin("origin")
            .collector("collector")
            .project_id("project_id")
            .project_name("project_name")
            .definition_id("definition_id")
            .definition_name("definition_name")
            .build_number("build_number")
            .status(BuildStatus::Unknown)
            .branch("branch")
            .url("https://dummy")
            .started_at(1578819921)
            .finished_at(Some(1578820921))
    }

    pub fn build(&self) -> Result<Build, String> {
        let build_id = Clone::clone(self.build_id.as_ref().ok_or("Build ID is missing")?);
        let provider = Clone::clone(self.provider.as_ref().ok_or("Build provider is missing")?);
        let origin = Clone::clone(self.origin.as_ref().ok_or("Origin is missing")?);
        let collector = Clone::clone(self.collector.as_ref().ok_or("Collector is missing")?);
        let project_id = Clone::clone(self.project_id.as_ref().ok_or("Project ID is missing")?);
        let project_name = Clone::clone(
            self.project_name
                .as_ref()
                .ok_or("Project Name is missing")?,
        );
        let definition_id = Clone::clone(
            self.definition_id
                .as_ref()
                .ok_or("Definition ID is missing")?,
        );
        let definition_name = Clone::clone(
            self.definition_name
                .as_ref()
                .ok_or("Definition name is missing")?,
        );
        let build_number = Clone::clone(
            self.build_number
                .as_ref()
                .ok_or("Build number is missing")?,
        );
        let status = Clone::clone(self.status.as_ref().ok_or("Build status is missing")?);
        let branch = Clone::clone(self.branch.as_ref().ok_or("Branch is missing")?);
        let url = Clone::clone(self.url.as_ref().ok_or("Url is missing")?);
        let started_at = Clone::clone(self.started_at.as_ref().ok_or("Start time is missing")?);
        let finished_at = Clone::clone(self.finished_at.as_ref().ok_or("Finish time is missing")?);

        // Generate a hash that represents the build.
        let mut hasher = DefaultHasher::new();
        provider.hash(&mut hasher);
        origin.hash(&mut hasher);
        project_id.hash(&mut hasher);
        definition_id.hash(&mut hasher);
        branch.hash(&mut hasher);
        build_id.hash(&mut hasher);
        let id = hasher.finish();

        // Generate a hash that represents the build
        // definition (partition) of the build, not the build itself.
        let mut hasher = DefaultHasher::new();
        provider.hash(&mut hasher);
        origin.hash(&mut hasher);
        project_id.hash(&mut hasher);
        definition_id.hash(&mut hasher);
        branch.hash(&mut hasher);
        let partition = hasher.finish();

        Ok(Build {
            id,
            partition,
            build_id,
            provider,
            origin,
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
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildStatus {
    Unknown,
    // Success,
    // Failed,
    // Running,
    // Canceled,
    // Queued,
}