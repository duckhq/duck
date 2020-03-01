use crate::DuckResult;

pub trait ConfigurationHandle {
    fn reload() -> DuckResult<Option<Configuration>>;
}

pub struct Configuration {
}