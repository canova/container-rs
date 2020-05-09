use crate::fs::FILE_SYSTEM_ROOT;
use crate::Result;
use flate2::read::GzDecoder;
use futures::future;
use reqwest;
use reqwest::header::ACCEPT;
use reqwest::ResponseBuilderExt;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;
use tar::Archive;
use tempfile::Builder;
use tempfile::TempDir;
use tokio;

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

  pub async fn get(mut self) -> Result<String> {
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

    // self.unwrap_to_file(layers).await?;
    Ok("".to_string())
  }

  async fn auth(&mut self) -> Result<()> {
    let auth_url = format!(
      "https://auth.docker.io/token?scope=repository:{image}:pull&service=registry.docker.io",
      image = self.image_name.as_ref().unwrap()
    );

    let res: DockerAuthResult = reqwest::get(&auth_url).await?.json().await?;
    self.auth_token = Some(res.token);
    info!("Docker auth token: {:?}", self.auth_token);
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
  ) -> Result<Vec<tokio::fs::File>> {
    info!("Getting the image layers from docker registry.");
    let mut file_system_path = PathBuf::from(FILE_SYSTEM_ROOT);
    file_system_path.push("images");
    file_system_path.push(self.image_name.as_ref().unwrap().replace("/", "_"));

    if !file_system_path.exists() {
      fs::create_dir_all(&file_system_path).expect("Failed to create the image dir");
    }

    let client = reqwest::Client::new();
    let mut files = future::join_all(manifest.layers.iter().map(|layer| {
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
        info!("path segments: {:?}", file.url().path_segments());
        let fname = file
          .url()
          .path_segments()
          .and_then(|mut segments| {
            let size = segments.clone().count();
            segments.nth(size - 2)
          })
          .and_then(|name| if name.is_empty() { None } else { Some(name) })
          .unwrap_or("tmp.bin");
        info!("file to download: '{}'", format!("{}.tar.gz", fname));
        let fname = file_system_path.join(format!("{}.tar.gz", fname));
        info!("will be located under: '{:?}'", fname);
        tokio::fs::File::create(fname).await?
      };

      tokio::io::copy(&mut &*file.bytes().await?, &mut dest).await?;
      file_objects.push(dest);
    }

    Ok(file_objects)
  }

  // async fn unwrap_to_file(&self, layers: Vec<tokio::fs::File>) -> Result<()> {
  //   info!("Starting to unwrap the docker image layers");
  //   let mut i: u32 = 1;
  //   for layer in layers {
  //     let mut file_system_path = PathBuf::from(FILE_SYSTEM_ROOT);
  //     file_system_path.push("images");
  //     file_system_path.push(self.image_name.as_ref().unwrap().replace("/", "_"));
  //     file_system_path.push("unzipped");
  //     if !file_system_path.exists() {
  //       fs::create_dir_all(&file_system_path).expect("Failed to create the image dir");
  //     }
  //     let mut std_file = layer.into_std().await;
  //     i += 1;
  //     info!("std_file: {:?}", std_file);
  //     let mut archive = Archive::new(GzDecoder::new(&std_file));
  //     info!("Unpacking tar {:?} to {:?}", std_file, file_system_path);
  //     archive.unpack(&file_system_path).unwrap();
  //   }

  //   Ok(())
  // }
}
