#[macro_use]
extern crate log;

mod cgroups;
mod container;

use container::Container;
use std::env;

/// It requires root privileges. Run the container ideally with `run.sh`.
/// Example command: `sudo run.sh run bash`.
fn main() {
    pretty_env_logger::init_timed();
    let args: Vec<_> = env::args().collect();
    info!("args: {:?}", args);
    assert!(args.len() > 1, "Expected a command but not found");

    match args[1].as_str() {
        "run" => run(&args[2..]),
        _ => unimplemented!(),
    }

    info!("exited!");
}

/// Run the main process with the given argument.
/// That function creates the child container process.
fn run(args: &[String]) {
    // Run the container process. This should initialize the process inside
    // and return the container process to us.
    let container = Container::new(args);
    // Wait for the container process.
    // TODO: support the deteched state.
    let status = container.wait();
    info!(
        "child process pid: {} status: {:?}",
        i32::from(container.pid),
        status
    );

    cleanup();
}

/// Cleanup function after the container process. This should handle only the
/// cgroups for now.
fn cleanup() {
    cgroups::deinit();
}
