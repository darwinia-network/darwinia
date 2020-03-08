#![cfg(unix)]
#![allow(dead_code)]

use assert_cmd::cargo::cargo_bin;
use nix::sys::signal::{kill, Signal::SIGINT};
use nix::unistd::Pid;
use std::{convert::TryInto, process::Command};
use std::{
	path::Path,
	process::{Child, ExitStatus},
	thread,
	time::Duration,
};

/// Wait for the given `child` the given number of `secs`.
///
/// Returns the `Some(exit status)` or `None` if the process did not finish in the given time.
pub fn wait_for(child: &mut Child, secs: usize) -> Option<ExitStatus> {
	for i in 0..secs {
		match child.try_wait().unwrap() {
			Some(status) => {
				if i > 5 {
					eprintln!("Child process took {} seconds to exit gracefully", i);
				}
				return Some(status);
			}
			None => thread::sleep(Duration::from_secs(1)),
		}
	}
	eprintln!("Took too long to exit (> {} seconds). Killing...", secs);
	let _ = child.kill();
	child.wait().unwrap();

	None
}

/// Run the node for a while (30 seconds)
pub fn run_dev_node_for_a_while(base_path: &Path) {
	let mut cmd = Command::new(cargo_bin("substrate"));

	let mut cmd = cmd.args(&["--dev"]).arg("-d").arg(base_path).spawn().unwrap();

	// Let it produce some blocks.
	thread::sleep(Duration::from_secs(30));
	assert!(cmd.try_wait().unwrap().is_none(), "the process should still be running");

	// Stop the process
	kill(Pid::from_raw(cmd.id().try_into().unwrap()), SIGINT).unwrap();
	assert!(wait_for(&mut cmd, 40).map(|x| x.success()).unwrap_or_default());
}
