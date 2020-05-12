use flate2::read::GzDecoder;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use tar::Archive;

const FILE_SYSTEM_ROOT: &str = "/var/container_rs";

pub struct FileSystem {
  pub container_id: String,
  pub path: PathBuf,
}

impl FileSystem {
  pub fn new(args: &clap::ArgMatches, container_id: String) -> Self {
    ensure_container_folder_exists(&container_id);
    let image = args.value_of("image").unwrap();
    let path = untar(image, &container_id);

    FileSystem { container_id, path }
  }
}

impl Drop for FileSystem {
  fn drop(&mut self) {
    info!("Dropping the FileSystem and its folder");
    fs::remove_dir_all(&self.path).expect("Failed to remove the filesystem directory");
  }
}

/// Either untar a single tarball or multiple layers of tarballs
fn untar(image: &str, container_id: &str) -> PathBuf {
  let mut file_system_path = PathBuf::from(FILE_SYSTEM_ROOT);
  file_system_path.push(container_id);

  if image.find(".tar").is_some() {
    untar_single(&image, &file_system_path);
  } else {
    let image_path = get_image_path(&image);

    info!("{:?}", image_path);
    if image_path.exists() && image_path.is_dir() {
      for entry in fs::read_dir(image_path).unwrap() {
        let entry = entry.unwrap();
        untar_single(entry.path().to_str().unwrap(), &file_system_path);
      }
    } else {
      error!("Image could not found!");
    }
  }

  file_system_path
}

/// Untar single tarball.
fn untar_single(image: &str, file_system_path: &PathBuf) {
  let now = Instant::now();
  let fs_tar_path = PathBuf::from(image).canonicalize().unwrap();
  let file = File::open(fs_tar_path).unwrap();

  info!("Unpacking tar {:?} to {:?}", file, file_system_path);
  let mut archive = Archive::new(GzDecoder::new(&file));
  archive.unpack(&file_system_path).unwrap();
  info!("Unpacked the file system tar ball in {:.2?}", now.elapsed());
}

fn ensure_container_folder_exists(container_id: &str) {
  let mut path = get_file_system_root_path();
  if !path.exists() {
    fs::create_dir(&path).expect("Failed to create the root file system dir");
  }
  path.push(container_id);
  if !path.exists() {
    fs::create_dir(&path).expect("Failed to create the root/fs file system dir");
  }
}

pub fn get_file_system_root_path() -> PathBuf {
  PathBuf::from(FILE_SYSTEM_ROOT)
}

pub fn get_images_path() -> PathBuf {
  let mut path = get_file_system_root_path();
  path.push("images");
  path
}

pub fn get_image_path(image: &str) -> PathBuf {
  let mut path = get_images_path();
  path.push(&image.replace("/", "_"));
  path
}
