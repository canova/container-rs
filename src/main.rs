#[macro_use]
extern crate log;

mod cgroups;
mod container;

use container::Container;
use std::env;

// ./container-rs run ls
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

fn run(args: &[String]) {
    let container = Container::new(args);
    let status = container.wait();
    info!(
        "child process pid: {} status: {:?}",
        i32::from(container.pid),
        status
    );

    cgroups::deinit();
}
