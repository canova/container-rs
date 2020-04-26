use std::fs;
use std::path::PathBuf;
use std::process;

/// Initialize the cgroups inside the container process.
pub fn init() {
  let mut cgroups = PathBuf::from("/sys/fs/cgroup/");
  assert!(cgroups.exists(), "Failed to locate cgroups");
  cgroups.push("pids");
  assert!(cgroups.exists(), "Failed to locate pids");
  cgroups.push("canova_test");

  if cgroups.exists() {
    // Shouldn't happen
    warn!("Unexecped existing cgroup");
  } else {
    fs::create_dir(&cgroups).expect("Failed to create the cgroup");
  }

  let pids_max = cgroups.join("pids.max");
  fs::write(pids_max, b"20").expect("Failed to write the pids.max");

  let cgroup_procs = cgroups.join("cgroup.procs");
  fs::write(cgroup_procs, process::id().to_string().as_bytes())
    .expect("Failed to attach the process to the cgroup");
}

/// Deinitialize the cgroups after container process being destroyed.
pub fn deinit() {
  let mut cgroups = PathBuf::from("/sys/fs/cgroup/");
  assert!(cgroups.exists(), "Failed to locate cgroups");
  cgroups.push("pids");
  assert!(cgroups.exists(), "Failed to locate pids");
  cgroups.push("canova_test");
  info!("cleaning up cgroups: {:?}", cgroups);
  fs::remove_dir(cgroups).expect("Failed to remove the cgroup");
}
