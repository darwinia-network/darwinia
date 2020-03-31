#![cfg(unix)]

use assert_cmd::cargo::cargo_bin;
use std::process::{Command, Stdio};
use tempfile::tempdir;

mod common;

#[test]
fn factory_works() {
	let base_path = tempdir().expect("could not create a temp dir");

	let status = Command::new(cargo_bin("darwinia"))
		.stdout(Stdio::null())
		.args(&["factory", "--dev", "-d"])
		.arg(base_path.path())
		.status()
		.unwrap();
	assert!(status.success());

	// Make sure that the `dev` chain folder exists & `db`
	assert!(base_path.path().join("chains/dev/").exists());
	assert!(base_path.path().join("chains/dev/db").exists());
}
