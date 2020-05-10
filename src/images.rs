use crate::fs::FILE_SYSTEM_ROOT;
use crate::Result;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;

///
pub async fn images(args: &clap::ArgMatches<'static>) -> Result<()> {
  if let Some(remove) = args.value_of("remove") {
    return Ok(remove_image(remove).await?);
  }

  list_images().await?;
  Ok(())
}

async fn remove_image(image: &str) -> Result<()> {
  if image.is_empty() {
    error!("Image name is not given");
    return Ok(());
  }

  let mut image_path = PathBuf::from(FILE_SYSTEM_ROOT);
  image_path.push("images");
  image_path.push(&image.replace("/", "_"));

  if image_path.exists() {
    fs::remove_dir_all(&image_path)?;
    info!("Deleted the image {:?}", image_path);
  } else {
    error!("Image does not exist");
  }
  Ok(())
}

async fn list_images() -> Result<()> {
  info!("Listing the images");
  let mut stdout = io::stdout();
  let mut images_path = PathBuf::from(FILE_SYSTEM_ROOT);
  images_path.push("images");

  for image in fs::read_dir(images_path)? {
    writeln!(&mut stdout, "{}", image?.file_name().into_string().unwrap())?;
  }
  Ok(())
}
