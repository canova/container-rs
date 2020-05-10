use crate::fs::{get_image_path, get_images_path};
use crate::Result;
use std::fs;
use std::io;
use std::io::Write;

/// Main entry for the images subcommand
/// TODO: Move all the subcommands into their own directory.
pub fn images(args: &clap::ArgMatches<'static>) -> Result<()> {
  if let Some(remove) = args.value_of("remove") {
    return Ok(remove_image(remove)?);
  }

  list_images()?;
  Ok(())
}

fn remove_image(image: &str) -> Result<()> {
  if image.is_empty() {
    error!("Image name is not given");
    // FIXME: Create a new error type and return it instead.
    return Ok(());
  }

  let image_path = get_image_path(&image);
  if image_path.exists() {
    fs::remove_dir_all(&image_path)?;
    info!("Deleted the image {:?}", image_path);
  } else {
    error!("Image does not exist");
    // FIXME: Create a new error type and return it instead.
  }
  Ok(())
}

fn list_images() -> Result<()> {
  info!("Listing the images");
  let mut stdout = io::stdout();
  let images_path = get_images_path();

  for image in fs::read_dir(images_path)? {
    writeln!(&mut stdout, "{}", image?.file_name().into_string().unwrap())?;
  }
  Ok(())
}
