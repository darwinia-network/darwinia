#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::process::Command;
use tempfile::tempdir;

mod common;

#[test]
fn inspect_works() {
	let base_path = tempdir().expect("could not create a temp dir");

	common::run_dev_node_for_a_while(base_path.path());

	let status = Command::new(cargo_bin("darwinia"))
		.args(&["inspect", "--dev", "--pruning", "archive", "-d"])
		.arg(base_path.path())
		.args(&["block", "1"])
		.status()
		.unwrap();
	assert!(status.success());
}
