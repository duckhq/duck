use url::Url;

use crate::builds::BuildStatus;
use crate::config::HueConfiguration;
use crate::utils::http::{HttpClient, HttpRequestBuilder, HttpResponse};
use crate::utils::{colors::Rgb, DuckResult};

pub struct HueClient {
    brightness: u8,
    url: Url,
    username: String,
    lights: Vec<String>,
}

impl HueClient {
    pub fn new(config: &HueConfiguration) -> Self {
        HueClient {
            brightness: match config.brightness {
                Option::Some(b) => b,
                Option::None => 255,
            },
            url: Url::parse(&config.hub_url[..]).unwrap(),
            username: config.username.clone(),
            lights: config.lights.clone(),
        }
    }

    pub fn turn_off(&self, client: &impl HttpClient) -> DuckResult<()> {
        self.set_light_state(client, format!("{{\"on\": {on} }}", on = false))?;
        Ok(())
    }

    pub fn set_state(&self, client: &impl HttpClient, status: BuildStatus) -> DuckResult<()> {
        if let Some((x, y)) = Self::get_cie_coordinates(&status) {
            self.set_light_state(
                client,
                format!(
                    "{{\"alert\":\"{alert}\",\"xy\":[{x},{y}],\"on\":{on},\"bri\":{brightness}}}",
                    alert = match status {
                        BuildStatus::Failed => "select",
                        _ => "none",
                    },
                    x = x,
                    y = y,
                    brightness = self.brightness,
                    on = true
                ),
            )?;
        }
        Ok(())
    }

    fn get_cie_coordinates(status: &BuildStatus) -> Option<(f32, f32)> {
        return match status {
            BuildStatus::Success => Some(Rgb::new(0, 255, 0).to_cie_coordinates()),
            BuildStatus::Failed => Some(Rgb::new(255, 0, 0).to_cie_coordinates()),
            BuildStatus::Running => Some(Rgb::new(127, 200, 255).to_cie_coordinates()),
            _ => None,
        };
    }

    fn set_light_state(&self, client: &impl HttpClient, body: String) -> DuckResult<()> {
        for light in &self.lights {
            let url = format!(
                "{url}api/{username}/lights/{id}/state",
                url = self.url,
                username = self.username,
                id = light
            );

            let mut builder = HttpRequestBuilder::put(url);
            builder.add_header("Content-Type", "application/json");
            builder.add_header("Accept", "application/json");
            builder.set_body(body.clone());

            let response = client.send(&builder)?;
            if !response.status().is_success() {
                return Err(format_err!(
                    "Could not update state for light '{id}' ({status})",
                    id = light,
                    status = response.status()
                ));
            }
        }

        Ok(())
    }
}
