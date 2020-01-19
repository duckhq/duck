use failure::Error;

pub mod colors;
pub mod date;

pub type DuckResult<T> = Result<T, Error>;
