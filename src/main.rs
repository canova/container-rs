#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod cgroups;
mod container;
mod fs;

use clap::{App, Arg, SubCommand};
use container::Container;
use std::env;

/// It requires root privileges. Run the container ideally with `run.sh`.
fn main() {
    pretty_env_logger::init_timed();

    // Parse command line arguments.
    let matches = App::new("container-rs")
        .version(crate_version!())
        .author(crate_authors!())
        .about("A container runtime written in Rust")
        .subcommand(
            SubCommand::with_name("run")
                .about("Run a container")
                .args(&[
                    // TODO: Add cgroup arguments here
                    Arg::with_name("pids.max")
                        .help("Limit the container processes (set -1 for unlimited)")
                        .long("pids.max")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name("file_sytem")
                        .help("A filesystem to run inside the container")
                        .required(true)
                        .takes_value(true),
                    Arg::with_name("command")
                        .help("A command to run inside the container")
                        .required(true)
                        .takes_value(true),
                    Arg::with_name("command_args")
                        .multiple(true)
                        .help("Arguments of the command to run inside the container")
                        .required(false),
                ]),
        )
        .get_matches();

    info!("args: {:?}", matches);

    match matches.subcommand_name() {
        Some("run") => run(matches
            .subcommand_matches("run")
            .expect("Failed to get subcommand matches")),
        None => panic!("Expected a command but not found"),
        _ => unimplemented!(),
    }

    info!("exited!");
}

/// Run the main process with the given argument.
/// That function creates the child container process.
fn run(args: &clap::ArgMatches) {
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
