use nix::mount::{mount, umount, MsFlags};
use nix::sched::{clone, unshare, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{chroot, sethostname, Pid};
use std::env::{current_dir, set_current_dir};
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};

pub struct Container {
  pub pid: Pid,
}

impl Container {
  pub fn new(args: &[String]) -> Self {
    // Stack creation
    const STACK_SIZE: usize = 1024 * 1024;
    let stack: &mut [u8; STACK_SIZE] = &mut [0; STACK_SIZE];
    // Callback for child process
    let callback = Box::new(|| child(args));

    let flags = CloneFlags::CLONE_NEWNS
      | CloneFlags::CLONE_NEWPID
      | CloneFlags::CLONE_NEWCGROUP
      | CloneFlags::CLONE_NEWUTS
      | CloneFlags::CLONE_NEWIPC
      | CloneFlags::CLONE_NEWNET;
    let pid = clone(callback, stack, flags, Some(Signal::SIGCHLD as i32))
      .expect("Container process creation failed!");

    Container { pid }
  }

  pub fn wait(&self) -> WaitStatus {
    waitpid(self.pid, None).expect("Failed to wait the container process")
  }
}

fn child(args: &[String]) -> isize {
  info!("Child process pid: {}", process::id());
  unshare(CloneFlags::CLONE_NEWNS).expect("Failed to unshare");
  assert!(args.is_empty(), "Expected a command but not found");

  cgroup();

  // Set the hostname
  sethostname("container").expect("Failed to set the hostname");

  // Change the root and set the working directory to it.
  let mut dir = current_dir().expect("Failed to get the current dir");
  dir.push("new_ubuntu");
  chroot(&dir).expect("Failed to set root directory");
  set_current_dir("/").expect("Failed to set the current dir");

  // Mount the /proc
  const NONE: Option<&'static [u8]> = None;
  mount(Some("proc"), "proc", Some("proc"), MsFlags::empty(), NONE)
    .expect("Failed to mount the /proc");

  let status = Command::new(&args[0])
    .args(&args[1..])
    .status()
    .expect("Failed to create child process inside the container");

  // Unount the /proc
  umount("proc").expect("Failed to unmount the /proc");

  info!("Child process status inside the container: {}", status);
  0
}

fn cgroup() {
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
