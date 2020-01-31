use crate::utils::DuckResult;
use regex::*;

static VARIABLE_REGEX: &str = r"\$\{(?P<VARIABLE>[A-Z_a-z0-1]+)\}";

pub trait VariableProvider {
    fn get_variable(&self, name: &str) -> DuckResult<String>;
}

//////////////////////////////////////////////////////////////////////
// The expander
//////////////////////////////////////////////////////////////////////

pub struct Expander<'a> {
    provider: &'a dyn VariableProvider,
    regex: Regex,
}

impl<'a> Expander<'a> {
    pub fn new(provider: &'a impl VariableProvider) -> Self {
        Self {
            provider,
            regex: Regex::new(VARIABLE_REGEX).unwrap(),
        }
    }

    pub fn expand<T: Into<String>>(&self, field: T) -> DuckResult<String> {
        let mut text = field.into();
        for capture in self.regex.captures_iter(&text.clone()[..]) {
            let variable = capture.name("VARIABLE").unwrap().as_str();
            let value = self.provider.get_variable(variable)?;
            text = text.replace(&format!("${{{}}}", variable)[..], &value[..]);
        }
        return Ok(text);
    }
}

//////////////////////////////////////////////////////////////////////
// Variable providers
//////////////////////////////////////////////////////////////////////

pub struct EnvironmentVariableProvider {}
impl EnvironmentVariableProvider {
    pub fn new() -> Self {
        Self {}
    }
}
impl VariableProvider for EnvironmentVariableProvider {
    fn get_variable(&self, name: &str) -> DuckResult<String> {
        let value = std::env::var(name);
        match value {
            Result::Ok(v) => Ok(v),
            Result::Err(_) => Err(format_err!(
                "Environment variable '{}' has not been set.",
                name
            )),
        }
    }
}

#[cfg(test)]
pub struct TestVariableProvider {
    lookup: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl VariableProvider for TestVariableProvider {
    fn get_variable(&self, name: &str) -> DuckResult<String> {
        let foo = self.lookup.get(name);
        match foo {
            Option::Some(t) => Ok(t.clone()),
            Option::None => Err(format_err!(
                "Environment variable '{}' has not been set.",
                name
            )),
        }
    }
}

#[cfg(test)]
impl TestVariableProvider {
    pub fn new() -> Self {
        Self {
            lookup: std::collections::HashMap::new(),
        }
    }

    pub fn add<T: Into<String>>(&mut self, key: T, value: T) {
        self.lookup.insert(key.into(), value.into());
    }
}
