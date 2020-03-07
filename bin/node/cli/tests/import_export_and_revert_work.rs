#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::{fs, process::Command};
use tempfile::tempdir;

mod common;

#[test]
fn import_export_and_revert_work() {
	let base_path = tempdir().expect("could not create a temp dir");
	let exported_blocks = base_path.path().join("exported_blocks");

	common::run_dev_node_for_a_while(base_path.path());

	let status = Command::new(cargo_bin("substrate"))
		.args(&["export-blocks", "--dev", "--pruning", "archive", "-d"])
		.arg(base_path.path())
		.arg(&exported_blocks)
		.status()
		.unwrap();
	assert!(status.success());

	let metadata = fs::metadata(&exported_blocks).unwrap();
	assert!(metadata.len() > 0, "file exported_blocks should not be empty");

	let _ = fs::remove_dir_all(base_path.path().join("db"));

	let status = Command::new(cargo_bin("substrate"))
		.args(&["import-blocks", "--dev", "--pruning", "archive", "-d"])
		.arg(base_path.path())
		.arg(&exported_blocks)
		.status()
		.unwrap();
	assert!(status.success());

	let status = Command::new(cargo_bin("substrate"))
		.args(&["revert", "--dev", "--pruning", "archive", "-d"])
		.arg(base_path.path())
		.status()
		.unwrap();
	assert!(status.success());
}
