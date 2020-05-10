mod docker;

use crate::Result;
use async_trait::async_trait;

pub use self::docker::DockerRegistry;

#[async_trait]
pub trait Registry {
  type Output;
  fn new() -> Self::Output;
  fn image_name(&mut self, image_name: String) -> &Self::Output;
  async fn get(mut self) -> Result<()>;
}
