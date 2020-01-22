use crate::utils::DuckResult;
use chrono::DateTime;

pub static TEAMCITY_FORMAT: &str = "%Y%m%dT%H%M%S%z";
pub static AZURE_DEVOPS_FORMAT: &str = "%+";
pub static OCTOPUS_DEPLOY_FORMAT: &str = "%+";

pub fn to_iso8601(input: &str, pattern: &str) -> DuckResult<String> {
    match DateTime::parse_from_str(input, pattern) {
        Ok(res) => Ok(res.format("%Y-%m-%dT%H:%M:%S%:z").to_string()),
        Err(e) => Err(format_err!("Could not parse date. {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_teamcity_format() {
        let result = to_iso8601("20191230T091041+0100", TEAMCITY_FORMAT).unwrap();
        assert_eq!("2019-12-30T09:10:41+01:00", result);
    }

    #[test]
    fn should_parse_azure_devops_format() {
        let result = to_iso8601("2020-01-12T09:05:21.0733795Z", AZURE_DEVOPS_FORMAT).unwrap();
        assert_eq!("2020-01-12T09:05:21+00:00", result);
    }

    #[test]
    fn should_parse_octopus_deploy_format() {
        let result = to_iso8601("2018-09-04T14:48:22.534+02:00", OCTOPUS_DEPLOY_FORMAT).unwrap();
        assert_eq!("2018-09-04T14:48:22+02:00", result);
    }
}
