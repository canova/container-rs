use crate::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub struct DockerRegistry {
  image_name: Option<String>,
  registry_url: String,
  auth_token: Option<String>,
}

// TODO: Use an error crate for better error handling.
#[derive(Debug)]
pub enum DockerRegistryError {
  ImageNameNotGiven,
}

impl std::fmt::Display for DockerRegistryError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Image name is not given")
  }
}

impl Error for DockerRegistryError {
  fn description(&self) -> &str {
    "Image name is not given"
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct DockerAuthResult {
  token: String,
}

impl DockerRegistry {
  pub fn new() -> Self {
    DockerRegistry {
      image_name: None,
      registry_url: "https://registry-1.docker.io/v2/".to_string(),
      auth_token: None,
    }
  }

  pub fn image_name(&mut self, image_name: String) -> &Self {
    self.image_name = Some(image_name);
    self
  }

  pub async fn get(self) -> Result<String> {
    if self.image_name.is_none() {
      return Err(Box::new(DockerRegistryError::ImageNameNotGiven));
    }

    // TODO: Get the image from registry
    // download_docker_image();

    Ok("".to_string())
  }

  fn download_docker_image(&self) {}

  async fn auth(&mut self) -> Result<()> {
    let auth_url = format!(
      "https://auth.docker.io/token?scope=repository:{image}:pull&service=registry.docker.io",
      image = self.image_name.as_ref().unwrap()
    );

    let res: DockerAuthResult = reqwest::get(auth_url).await?.json()?.await?;;
    self.auth_token = res.token;
    info!("Docker auth token: {}", self.auth_token)
    Ok(())
  }
}
