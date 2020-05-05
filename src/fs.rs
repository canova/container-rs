use flate2::read::GzDecoder;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;

const FILE_SYSTEM_ROOT: &str = "/var/container_rs";

pub struct FileSystem {
  pub path: PathBuf,
}

impl FileSystem {
  pub fn new(args: &clap::ArgMatches) -> Self {
    ensure_root_folder_exists();
    let file_system_tar = args.value_of("file_sytem").unwrap();
    let path = untar(file_system_tar);

    FileSystem { path }
  }
}

impl Drop for FileSystem {
  fn drop(&mut self) {
    info!("Dropping the FileSystem and its folder");
    fs::remove_dir_all(&self.path).expect("Failed to remove the filesystem directory");
  }
}

fn untar(fs_tarball: &str) -> PathBuf {
  let fs_tar_path = PathBuf::from(fs_tarball);
  let file_cann = fs_tar_path.canonicalize().unwrap();
  let file = File::open(file_cann).unwrap();
  let file_system_path = PathBuf::from(format!("{}/fs", FILE_SYSTEM_ROOT));

  let decompressed = GzDecoder::new(&file);
  let mut archive = Archive::new(decompressed);
  info!("Unpacking tar {:?} to {:?}", file, file_system_path);
  archive.unpack(&file_system_path).unwrap();
  file_system_path
}

fn ensure_root_folder_exists() {
  let mut path = PathBuf::from(FILE_SYSTEM_ROOT);
  if !path.exists() {
    fs::create_dir(&path).expect("Failed to create the root file system dir");
  }
  path.push("fs");
  if !path.exists() {
    fs::create_dir(&path).expect("Failed to create the root/fs file system dir");
  }
}
