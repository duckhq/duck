use crate::DuckResult;

pub trait ConfigurationHandle {
    fn has_changed(&self) -> DuckResult<bool>;
    fn load(&self) -> DuckResult<Configuration>;
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Configuration {
    pub title: Option<String>,
    pub interval: Option<u16>,
}
