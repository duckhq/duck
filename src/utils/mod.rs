use failure::Error;

pub mod colors;
pub mod date;
pub mod http;

pub type DuckResult<T> = Result<T, Error>;
