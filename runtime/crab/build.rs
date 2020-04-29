use wasm_builder_runner::WasmBuilder;

fn main() {
	WasmBuilder::new()
		.with_current_project()
		.with_wasm_builder_from_git(
			"https://github.com/paritytech/substrate.git",
			"8c672e107789ed10973d937ba8cac245404377e2",
		)
		.import_memory()
		.export_heap_base()
		.build()
}
