use std::path::PathBuf;
use structopt::StructOpt;

use crate::DuckResult;

pub const DEFAULT_CONFIG: &str = "config.json";

#[derive(StructOpt, Debug)]
pub struct Arguments {
    /// The configuration file
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = DEFAULT_CONFIG,
        env = "DUCK_CONFIG"
    )]
    pub config: PathBuf,
    /// The server address to bind to
    #[structopt(name = "bind", short, long, env = "DUCK_BIND")]
    server_address: Option<String>,
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments {
            config: PathBuf::from(DEFAULT_CONFIG),
            server_address: None,
        }
    }
}

/// Executes the run command
pub async fn execute(args: Arguments) -> DuckResult<()> {
    duck::run(args.config, args.server_address).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn default_arguments_should_have_correct_configuration_file() {
        // Given, When
        let args = Arguments::default();
        // When
        let config = args.config.to_str().unwrap();
        // Then
        assert_eq!("config.json", config);
    }
}
