use url::Url;

use crate::config::{HueConfiguration, Validate};
use crate::utils::DuckResult;

impl Validate for HueConfiguration {
    fn validate(&self) -> DuckResult<()> {
        if self.id.is_empty() {
            return Err(format_err!("Hue ID is empty."));
        }
        if let Err(e) = Url::parse(&self.hub_url[..]) {
            return Err(format_err!("Hue hub URL is invalid: {}", e));
        }
        if self.username.is_empty() {
            return Err(format_err!("Hue username is empty."));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Configuration;

    #[test]
    #[should_panic(expected = "Hue ID is empty.")]
    fn should_return_error_if_hue_id_is_empty() {
        let config = Configuration::from_json(
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "hue": {
                            "id": "",
                            "hubUrl": "https://localhost:5000",
                            "username": "vpBIFkq-2iWFvSLf62u1HvcmLbqbDf76N-CTom8b",
                            "lights": [ "3" ]
                        }             
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "Hue hub URL is invalid: relative URL without a base")]
    fn should_return_error_if_hue_hub_url_is_empty() {
        let config = Configuration::from_json(
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "hue": {
                            "id": "bar",
                            "hubUrl": "",
                            "username": "vpBIFkq-2iWFvSLf62u1HvcmLbqbDf76N-CTom8b",
                            "lights": [ "3" ]
                        }             
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }

    #[test]
    #[should_panic(expected = "Hue username is empty.")]
    fn should_return_error_if_hue_username_is_empty() {
        let config = Configuration::from_json(
            r#"
            { 
                "collectors": [ ],
                "observers": [
                    {
                        "hue": {
                            "id": "bar",
                            "hubUrl": "https://localhost:6000",
                            "username": "",
                            "lights": [ "3" ]
                        }             
                    }
                ]
            }
        "#,
        )
        .unwrap();
        config.validate().unwrap();
    }
}
