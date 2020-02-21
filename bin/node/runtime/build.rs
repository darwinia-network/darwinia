use std::{
	env, fs,
	path::Path,
	time::{SystemTime, UNIX_EPOCH},
};

use wasm_builder_runner::{build_current_project_with_rustflags, WasmBuilderSource};

fn main() {
	fs::write(
		&Path::new(&env::var_os("OUT_DIR").unwrap()).join("timestamp_now.rs"),
		&format!(
			"pub const NOW: u64 = {};",
			SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
		),
	)
	.unwrap();

	build_current_project_with_rustflags(
		"wasm_binary.rs",
		// TODO: update version
		WasmBuilderSource::Crates("1.0.8"),
		// This instructs LLD to export __heap_base as a global variable, which is used by the
		// external memory allocator.
		"-Clink-arg=--export=__heap_base",
	);
}
