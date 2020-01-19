use std::collections::HashSet;
use std::iter::FromIterator;

use log::{error, info};
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::{Client, ClientBuilder};
use url::Url;

use crate::builds::BuildStatus;
use crate::config::HueConfiguration;
use crate::observers::{Observation, Observer, ObserverInfo};
use crate::utils::colors::Rgb;
use crate::utils::DuckResult;

pub struct HueObserver {
    info: ObserverInfo,
    client: HueClient,
}

impl HueObserver {
    pub fn new(config: &HueConfiguration) -> Self {
        HueObserver {
            client: HueClient::new(config),
            info: ObserverInfo {
                id: config.id.clone(),
                enabled: match config.enabled {
                    None => true,
                    Some(e) => e,
                },
                collectors: match &config.collectors {
                    Option::None => Option::None,
                    Option::Some(collectors) => {
                        Some(HashSet::from_iter(collectors.iter().cloned()))
                    }
                },
            },
        }
    }
}

impl Observer for HueObserver {
    fn info(&self) -> &ObserverInfo {
        &self.info
    }

    fn observe(&self, observation: Observation) -> DuckResult<()> {
        match observation {
            Observation::DuckStatusChanged(status) => {
                info!(
                    "[{}] Setting light state to '{:?}'...",
                    self.info.id, status
                );
                self.client.set_state(status)?;
            }
            Observation::ShuttingDown => {
                info!("[{}] Turning off all lights...", self.info.id);
                self.client.turn_off()?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct HueClient {
    client: Client,
    brightness: u8,
    url: Url,
    username: String,
    lights: Vec<String>,
}

impl HueClient {
    pub fn new(config: &HueConfiguration) -> Self {
        HueClient {
            client: ClientBuilder::new().build().unwrap(),
            brightness: match config.brightness {
                Option::Some(b) => b,
                Option::None => 255,
            },
            url: Url::parse(&config.hub_url[..]).unwrap(),
            username: config.username.clone(),
            lights: config.lights.clone(),
        }
    }

    pub fn turn_off(&self) -> DuckResult<()> {
        self.set_light_state(format!("{{\"on\": {on} }}", on = false))?;
        Ok(())
    }

    pub fn set_state(&self, status: BuildStatus) -> DuckResult<()> {
        let (x, y) = HueClient::get_cie_coordinates(&status);
        self.set_light_state(format!(
            "{{\"alert\":\"{alert}\",\"xy\":[{x},{y}],\"on\":{on},\"bri\": {brightness} }}",
            alert = match status {
                BuildStatus::Failed => "select",
                _ => "none",
            },
            x = x,
            y = y,
            brightness = self.brightness,
            on = true
        ))
    }

    fn get_cie_coordinates(status: &BuildStatus) -> (f32, f32) {
        return match status {
            BuildStatus::Unknown => Rgb::new(255, 255, 255).to_cie_coordinates(),
            BuildStatus::Success => Rgb::new(0, 255, 0).to_cie_coordinates(),
            BuildStatus::Failed => Rgb::new(255, 0, 0).to_cie_coordinates(),
            BuildStatus::Running => Rgb::new(127, 200, 255).to_cie_coordinates(),
        };
    }

    fn set_light_state(&self, body: String) -> DuckResult<()> {
        for light in &self.lights {
            let url = format!(
                "{url}api/{username}/lights/{id}/state",
                url = self.url,
                username = self.username,
                id = light
            );

            let response = self
                .client
                .put(&url)
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "application/json")
                .body(body.clone())
                .send()?;

            if !response.status().is_success() {
                error!("Could not set state ({})!", response.status());
            }
        }

        Ok(())
    }
}
