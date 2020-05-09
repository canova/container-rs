use crate::cgroups;
use crate::fs::FileSystem;
use crate::registries::DockerRegistry;
use crate::Result;
use hex;
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{clone, unshare, CloneFlags};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{chroot, sethostname, Pid};
use sha2::digest::Digest;
use sha2::Sha256;
use std::env::{current_dir, set_current_dir};
use std::process::{self, Command};
use std::time::SystemTime;

pub struct Container {
  pub id: String,
  pub pid: Pid,
  pub file_system: FileSystem,
}

impl Container {
  /// Initialize a new container process and return it.
  pub async fn new(args: &clap::ArgMatches<'static>) -> Self {
    // Get the container ID as sha256 from the current timestamp.
    let mut hasher = Sha256::new();
    let unix_timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
      .to_be_bytes();

    hasher.input(unix_timestamp);
    let id = hex::encode(hasher.result());
    info!("Container id: {}", id);

    if let Some(registry) = args.value_of("registry") {
      let image = get_image(args).await.unwrap();
    }

    // Create a new filesystem and pass this into the container.
    // TODO: Remove the String clone by sending a reference.
    let file_system = FileSystem::new(args, id.clone());

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
    Container {
      id,
      pid,
      file_system,
    }
  }

  /// Wait for the container process until it's done.
  pub fn wait(&self) -> WaitStatus {
    waitpid(self.pid, None).expect("Failed to wait the container process")
  }
}

fn child(args: &clap::ArgMatches<'static>) -> isize {
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

async fn get_image(args: &clap::ArgMatches<'static>) -> Result<String> {
  let mut docker_registry = DockerRegistry::new();

  if let Some(image_name) = args.value_of("file_sytem") {
    docker_registry.image_name(image_name.to_string());
  }

  docker_registry.get().await
}
