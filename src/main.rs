#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

mod cgroups;
mod container;
mod fs;
mod images;
mod pull;
mod registries;

use crate::images::images;
use crate::pull::pull;
use clap::{App, Arg, SubCommand};
use container::Container;
use std::env;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// It requires root privileges. Run the container ideally with `run.sh`.
#[tokio::main]
async fn main() -> Result<()> {
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
                    Arg::with_name("registry")
                        .help("Registry we would like to use for images")
                        .long("registry")
                        .short("r")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name("image")
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
        .subcommand(
            SubCommand::with_name("pull")
                .about("Pull image from a registry")
                .args(&[
                    Arg::with_name("registry")
                        .help("Registry we would like to use for images")
                        .long("registry")
                        .short("r")
                        .takes_value(true)
                        .default_value("docker")
                        .required(false),
                    Arg::with_name("image")
                        .help("Image name that we want to pull")
                        .required(true)
                        .takes_value(true),
                    // Arg::with_name("tag")
                    //     .help("Tag that we want to pull")
                    //     .required(true)
                    //     .takes_value(true),
                ]),
        )
        .subcommand(
            SubCommand::with_name("images")
                .about("Get the list of the pulled images")
                .args(&[Arg::with_name("remove")
                    .help("Registry we would like to use for images")
                    .long("remove")
                    .short("r")
                    .takes_value(true)
                    .required(false)]),
        )
        .get_matches();

    info!("args: {:?}", matches);

    match matches.subcommand_name() {
        Some("run") => run(matches
            .subcommand_matches("run")
            .expect("Failed to get subcommand matches")),
        Some("pull") => {
            pull(
                matches
                    .subcommand_matches("pull")
                    .expect("Failed to get subcommand matches"),
            )
            .await?
        }
        Some("images") => images(
            matches
                .subcommand_matches("images")
                .expect("Failed to get subcommand matches"),
        )?,
        None => panic!("Expected a command but not found"),
        _ => unimplemented!(),
    }

    info!("exited!");
    Ok(())
}

/// Run the main process with the given argument.
/// That function creates the child container process.
fn run(args: &clap::ArgMatches<'static>) {
    // Run the container process. This should initialize the process inside
    // and return the container process to us.
    let container = Container::new(args);
    // Wait for the container process.
    // TODO: support the detached state.
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
