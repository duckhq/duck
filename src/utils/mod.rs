use failure::Error;

pub mod colors;
pub mod date;
pub mod http;
pub mod text;

pub type DuckResult<T> = Result<T, Error>;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
