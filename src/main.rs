mod container;

use container::Container;
use std::env;
use std::fs;
use std::path::PathBuf;

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
    let container = Container::new(args);
    let status = container.wait();
    println!(
        "child process pid: {} status: {:?}",
        i32::from(container.pid),
        status
    );

    cleanup_cgroup();
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
