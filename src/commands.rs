pub mod schema;
pub mod start;
pub mod validate;

#[cfg(windows)]
pub mod service;

pub const DEFAULT_CONFIG: &str = "config.json";
pub const ENV_CONFIG: &str = "DUCK_CONFIG";
pub const ENV_BINDING: &str = "DUCK_BIND";
