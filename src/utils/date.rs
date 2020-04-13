use chrono::DateTime;

use crate::DuckResult;

pub static TEAMCITY_FORMAT: &str = "%Y%m%dT%H%M%S%z";
pub static AZURE_DEVOPS_FORMAT: &str = "%+";
pub static GITHUB_FORMAT: &str = "%+";
pub static OCTOPUS_DEPLOY_FORMAT: &str = "%+";
pub static APPVEYOR_FORMAT: &str = "%+";
pub static DEBUGGER_FORMAT: &str = "%+";

pub fn to_timestamp(input: &str, pattern: &str) -> DuckResult<i64> {
    match DateTime::parse_from_str(input, pattern) {
        Ok(res) => Ok(res.timestamp()),
        Err(e) => Err(format_err!("Could not parse date. {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_teamcity_format() {
        let result = to_timestamp("20191230T091041+0100", TEAMCITY_FORMAT).unwrap();
        assert_eq!(1577693441, result);
    }

    #[test]
    fn should_parse_azure_devops_format() {
        let result = to_timestamp("2020-01-12T09:05:21.0733795Z", AZURE_DEVOPS_FORMAT).unwrap();
        assert_eq!(1578819921, result);
    }

    #[test]
    fn should_parse_octopus_deploy_format() {
        let result = to_timestamp("2018-09-04T14:48:22.534+02:00", OCTOPUS_DEPLOY_FORMAT).unwrap();
        assert_eq!(1536065302, result);
    }

    #[test]
    fn should_parse_github_format() {
        let result = to_timestamp("2020-02-01T20:43:16Z", GITHUB_FORMAT).unwrap();
        assert_eq!(1580589796, result);
    }

    #[test]
    fn should_parse_appveyor_format() {
        let result = to_timestamp("2020-03-11T12:09:48.1638791+00:00", APPVEYOR_FORMAT).unwrap();
        assert_eq!(1583928588, result);
    }

    #[test]
    fn should_parse_debugger_format() {
        let result = to_timestamp("2020-04-13T17:04:23.6101884+02:00", DEBUGGER_FORMAT).unwrap();
        assert_eq!(1586790263, result);
    }
}
