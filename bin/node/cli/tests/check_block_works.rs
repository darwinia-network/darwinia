#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::process::Command;
use tempfile::tempdir;

mod common;

#[test]
fn check_block_works() {
	let base_path = tempdir().expect("could not create a temp dir");

	common::run_dev_node_for_a_while(base_path.path());

	let status = Command::new(cargo_bin("substrate"))
		.args(&["check-block", "--dev", "--pruning", "archive", "-d"])
		.arg(base_path.path())
		.arg("1")
		.status()
		.unwrap();
	assert!(status.success());
}
