use std::{
	env, fs,
	path::Path,
	time::{SystemTime, UNIX_EPOCH},
};

use wasm_builder_runner::WasmBuilder;

fn main() {
	fs::write(
		&Path::new(&env::var_os("OUT_DIR").unwrap()).join("timestamp_now.rs"),
		&format!(
			"pub const NOW: u64 = {};",
			SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
		),
	)
	.unwrap();

	WasmBuilder::new()
		.with_current_project()
		.with_wasm_builder_from_crates("1.0.9")
		.export_heap_base()
		.import_memory()
		.build()
}
