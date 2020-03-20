use std::path::PathBuf;

use duck::DuckResult;
use structopt::StructOpt;

use crate::commands::{DEFAULT_CONFIG, ENV_BINDING, ENV_CONFIG};

///////////////////////////////////////////////////////////
// Arguments

#[derive(StructOpt, Debug)]
pub struct Arguments {
    /// The configuration file
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = DEFAULT_CONFIG,
        env = ENV_CONFIG
    )]
    pub config: PathBuf,
    /// The server address to bind to
    #[structopt(name = "bind", short, long, env = ENV_BINDING)]
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

///////////////////////////////////////////////////////////
// Command

pub async fn execute(args: Arguments) -> DuckResult<()> {
    duck::run(args.config, args.server_address).await
}

///////////////////////////////////////////////////////////
// Tests

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
        assert_eq!(DEFAULT_CONFIG, config);
    }

    #[test]
    pub fn default_arguments_should_have_correct_server_address() {
        // Given, When
        let args = Arguments::default();
        // When
        let server_address = args.server_address;
        // Then
        assert!(server_address.is_none());
    }
}
