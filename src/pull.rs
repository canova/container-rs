use crate::registries::{DockerRegistry, Registry};
use crate::Result;

/// Main entry for the pull subcommand
/// TODO: Move all the subcommands into their own directory.
pub async fn pull(args: &clap::ArgMatches<'static>) -> Result<()> {
  let registry = args.value_of("registry").unwrap();
  let mut registry = match registry {
    "docker" => DockerRegistry::new(),
    _ => unimplemented!(),
  };

  if let Some(image_name) = args.value_of("image") {
    registry.image_name(image_name.to_string());
  }
  registry.get().await
}
