fn main() {
	wasm_builder_runner::WasmBuilder::new()
		.with_current_project()
		.with_wasm_builder_from_crates("1.0.9")
		.export_heap_base()
		.import_memory()
		.build()
}
