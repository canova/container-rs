use crate::fs::FILE_SYSTEM_ROOT;
use crate::Result;
use futures::future;
use reqwest;
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use tempfile::Builder;
use tempfile::TempDir;
use tokio;
use tokio::fs::OpenOptions;

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

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct DockerManifestResult {
  schemaVersion: u32,
  mediaType: String,
  config: DockerManifestConfig,
  layers: Vec<DockerManifestLayer>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct DockerManifestConfig {
  mediaType: String,
  size: u32,
  digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct DockerManifestLayer {
  pub mediaType: String,
  size: u32,
  digest: String,
}

impl DockerRegistry {
  pub fn new() -> Self {
    DockerRegistry {
      image_name: None,
      registry_url: "https://registry-1.docker.io/v2".to_string(),
      auth_token: None,
    }
  }

  pub fn image_name(&mut self, image_name: String) -> &Self {
    self.image_name = Some(image_name);
    self
  }

  pub async fn get(mut self) -> Result<()> {
    info!("Getting the image from docker registry");
    if self.image_name.is_none() {
      return Err(Box::new(DockerRegistryError::ImageNameNotGiven));
    }

    if self.auth_token.is_none() {
      info!("No auth token found, getting the default token");
      self.auth().await?;
    }

    let tmp_dir = Builder::new().prefix("example").tempdir()?;
    let manifest = self.get_manifest().await?;
    let layers = self
      .download_docker_image_layers(&manifest, &tmp_dir)
      .await?;

    self.copy_to_images_dir(layers).await?;
    Ok(())
  }

  async fn auth(&mut self) -> Result<()> {
    let auth_url = format!(
      "https://auth.docker.io/token?scope=repository:{image}:pull&service=registry.docker.io",
      image = self.image_name.as_ref().unwrap()
    );

    let res: DockerAuthResult = reqwest::get(&auth_url).await?.json().await?;
    self.auth_token = Some(res.token);
    info!("Got the Docker auth token");
    Ok(())
  }

  async fn get_manifest(&self) -> Result<DockerManifestResult> {
    info!("Getting the image manifest from docker registry.");
    let url = format!(
      "{registry}/{image}/manifests/latest",
      registry = self.registry_url,
      image = self.image_name.as_ref().unwrap()
    );

    let client = reqwest::Client::new();

    let res = client
      .get(&url)
      .header(
        ACCEPT,
        "application/vnd.docker.distribution.manifest.v2+json",
      )
      .bearer_auth(self.auth_token.as_ref().unwrap())
      .send()
      .await?
      .json()
      .await?;

    info!("Got the manifest file.");
    Ok(res)
  }

  async fn download_docker_image_layers(
    &self,
    manifest: &DockerManifestResult,
    tmp_dir: &TempDir,
  ) -> Result<Vec<(PathBuf, tokio::fs::File)>> {
    info!("Getting the image layers from docker registry.");

    let client = reqwest::Client::new();
    let files = future::join_all(manifest.layers.iter().map(|layer| {
      let url = format!(
        "{registry}/{image}/blobs/{digest}",
        registry = self.registry_url,
        image = self.image_name.as_ref().unwrap(),
        digest = layer.digest
      );
      let client = &client;
      async move {
        client
          .get(&url)
          .bearer_auth(self.auth_token.as_ref().unwrap())
          .send()
          .await
      }
    }))
    .await;

    let mut file_objects = vec![];
    for file in files {
      let file = file?;
      let mut dest = {
        let file_path = file
          .url()
          .path_segments()
          .and_then(|mut segments| {
            let size = segments.clone().count();
            segments.nth(size - 2)
          })
          .and_then(|name| if name.is_empty() { None } else { Some(name) })
          .unwrap_or("tmp.bin");
        let file_path = tmp_dir.path().join(format!("{}.tar.gz", file_path));
        let file = OpenOptions::new()
          .read(true)
          .write(true)
          .create(true)
          .open(&file_path)
          .await?;
        (file_path, file)
      };

      tokio::io::copy(&mut &*file.bytes().await?, &mut dest.1).await?;
      file_objects.push(dest);
    }

    Ok(file_objects)
  }

  async fn copy_to_images_dir(&self, layers: Vec<(PathBuf, tokio::fs::File)>) -> Result<()> {
    info!("Starting to unwrap the docker image layers");
    let mut image_path = PathBuf::from(FILE_SYSTEM_ROOT);
    image_path.push("images");
    image_path.push(self.image_name.as_ref().unwrap().replace("/", "_"));
    if !image_path.exists() {
      fs::create_dir_all(&image_path).expect("Failed to create the image dir");
    } else {
      warn!("This image already exists");
    }

    // Copying the layers to their place.r!
    for mut layer in layers {
      let file_name = layer.0.file_name().unwrap();
      let dest_path = image_path.join(file_name);
      let mut dest = tokio::fs::File::create(dest_path).await?;

      tokio::io::copy(&mut layer.1, &mut dest).await?;
    }

    info!("Succesfully copied all the image layers");
    Ok(())
  }
}
