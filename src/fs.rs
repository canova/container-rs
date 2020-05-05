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
    let fs_path = args.value_of("file_sytem").unwrap();
    let path = untar(fs_path, &container_id);

    FileSystem { container_id, path }
  }
}

impl Drop for FileSystem {
  fn drop(&mut self) {
    info!("Dropping the FileSystem and its folder");
    fs::remove_dir_all(&self.path).expect("Failed to remove the filesystem directory");
  }
}

fn untar(fs_tarball: &str, container_id: &str) -> PathBuf {
  let now = Instant::now();
  let fs_tar_path = PathBuf::from(fs_tarball).canonicalize().unwrap();
  info!("Starting to unpack: {:?}", fs_tar_path);
  let file = File::open(fs_tar_path).unwrap();
  let mut file_system_path = PathBuf::from(FILE_SYSTEM_ROOT);
  file_system_path.push(container_id);

  let mut archive = Archive::new(GzDecoder::new(&file));
  info!("Unpacking tar {:?} to {:?}", file, file_system_path);
  archive.unpack(&file_system_path).unwrap();
  info!("Unpacked the file system tar ball in {:.2?}", now.elapsed());
  file_system_path
}

fn ensure_container_folder_exists(container_id: &str) {
  let mut path = PathBuf::from(FILE_SYSTEM_ROOT);
  if !path.exists() {
    fs::create_dir(&path).expect("Failed to create the root file system dir");
  }
  path.push(container_id);
  if !path.exists() {
    fs::create_dir(&path).expect("Failed to create the root/fs file system dir");
  }
}
