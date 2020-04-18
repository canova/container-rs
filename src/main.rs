use nix::mount::mount;
use nix::mount::umount;
use nix::mount::MsFlags;
use nix::sched;
use nix::sched::unshare;
use nix::sched::CloneFlags;
use nix::sys::signal::Signal;
use nix::sys::wait::waitpid;
use nix::unistd::chroot;
use nix::unistd::sethostname;
use std::env;
use std::env::current_dir;
use std::env::set_current_dir;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::process::Command;

// ./container-rs run ls
fn main() {
    let args: Vec<_> = env::args().collect();
    println!("args: {:?}", args);
    assert!(args.len() > 1, "Expected a command but not found");

    match args[1].as_str() {
        "run" => run(&args[2..]),
        _ => unimplemented!(),
    }

    println!("exited!");
}

fn run(args: &[String]) {
    // Stack creation
    const STACK_SIZE: usize = 1024 * 1024;
    let ref mut stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    // Callback for child process
    let callback = Box::new(|| child(args));

    let flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWCGROUP
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWNET;
    let pid = sched::clone(callback, stack, flags, Some(Signal::SIGCHLD as i32))
        .expect("Container process creation failed!");
    let status = waitpid(pid, None).expect("Failed to wait the container process");
    println!("child process pid: {} status: {:?}", i32::from(pid), status);

    cleanup_cgroup();
}

fn child(args: &[String]) -> isize {
    println!("Child process pid: {}", process::id());
    unshare(CloneFlags::CLONE_NEWNS).expect("Failed to unshare");
    assert!(args.len() > 0, "Expected a command but not found");

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

    println!("Child process status inside the container: {}", status);
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
        println!("Unexecped existing cgroup");
    } else {
        fs::create_dir(&cgroups).expect("Failed to create the cgroup");
    }

    let pids_max = cgroups.join("pids.max");
    fs::write(pids_max, "20".as_bytes()).expect("Failed to write the pids.max");

    let cgroup_procs = cgroups.join("cgroup.procs");
    fs::write(cgroup_procs, process::id().to_string().as_bytes())
        .expect("Failed to attach the process to the cgroup");
}

fn cleanup_cgroup() {
    let mut cgroups = PathBuf::from("/sys/fs/cgroup/");
    assert!(cgroups.exists(), "Failed to locate cgroups");
    cgroups.push("pids");
    assert!(cgroups.exists(), "Failed to locate pids");
    cgroups.push("canova_test");
    println!("cgroups: {:?}", cgroups);
    fs::remove_dir(cgroups).expect("Failed to remove the cgroup");
}
