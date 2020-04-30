use crate::cgroups;
use crate::filesys::FileSystem;
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{clone, unshare, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{chroot, sethostname, Pid};
use std::env::{current_dir, set_current_dir};
use std::process::{self, Command};

pub struct Container {
  pub pid: Pid,
  pub file_system: FileSystem,
}

impl Container {
  /// Initialize a new container process and return it.
  pub fn new(args: &clap::ArgMatches, file_system: FileSystem) -> Self {
    // Stack creation
    const STACK_SIZE: usize = 1024 * 1024;
    let stack: &mut [u8; STACK_SIZE] = &mut [0; STACK_SIZE];
    // Callback for child process
    let callback = Box::new(|| child(args));

    // Create the flags for the new container process. These flags
    // creates new namespaces and assigns them to the child process.
    let flags = CloneFlags::CLONE_NEWNS
      | CloneFlags::CLONE_NEWPID
      | CloneFlags::CLONE_NEWCGROUP
      | CloneFlags::CLONE_NEWUTS
      | CloneFlags::CLONE_NEWIPC
      | CloneFlags::CLONE_NEWNET;
    // Create the process with the clone syscall. Rust's Command struct
    // is not enough to create a container process because there is no
    // way to pass a clone flag.
    let pid = clone(callback, stack, flags, Some(Signal::SIGCHLD as i32))
      .expect("Container process creation failed!");

    // Return the container struct.
    Container { pid, file_system }
  }

  /// Wait for the container process until it's done.
  pub fn wait(&self) -> WaitStatus {
    waitpid(self.pid, None).expect("Failed to wait the container process")
  }
}

fn child(args: &clap::ArgMatches) -> isize {
  info!("Child process pid: {}", process::id());
  // Unshare the namespace
  unshare(CloneFlags::CLONE_NEWNS).expect("Failed to unshare");

  // Initialize the cgroups
  cgroups::init(args);

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

  // Create a new process with the given command from the arguments and wait
  // for it until it's done. Returns the status code of the process.
  let mut command = Command::new(
    &args
      .value_of("command")
      .expect("Failed to get the command argument"),
  );

  if let Some(args) = args.values_of("command_args") {
    info!("subcommand arguments: {:?}", args);
    command.args(args);
  }

  let status = command
    .status()
    .expect("Failed to create child process inside the container");

  // Unount the /proc
  umount("proc").expect("Failed to unmount the /proc");

  info!("Child process status inside the container: {}", status);
  0
}
