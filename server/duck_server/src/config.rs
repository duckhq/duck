use crate::DuckResult;

pub trait ConfigurationHandle {
    fn has_changed(&self) -> DuckResult<bool>;
    fn load(&self) -> DuckResult<Configuration>;
}

pub struct Configuration {
}