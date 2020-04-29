use wasm_builder_runner::WasmBuilder;

fn main() {
	WasmBuilder::new()
		.with_current_project()
		.with_wasm_builder_from_crates("1.0.9")
		.import_memory()
		.export_heap_base()
		.build()
}
