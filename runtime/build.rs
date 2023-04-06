fn main() {
    #[cfg(feature = "std")]
    {
        /// This code initializes a new WasmBuilder instance from the substrate_wasm_builder crate,
        /// which is used to build and compile wasm code for use in a Substrate runtime.
        ///
        /// The new() method initializes the builder with default settings. The with_current_project()
        /// method sets the current working directory as the project root, which is where the wasm
        /// code is located. The export_heap_base() method sets up the wasm memory and exports the
        /// base heap pointer. The import_memory() method imports the wasm memory instance from the
        /// Substrate node. The build() method then generates the final wasm binary code that is
        /// used to run the Substrate runtime.
        substrate_wasm_builder::WasmBuilder::new()
            .with_current_project()
            .export_heap_base()
            .import_memory()
            .build();
    }
}
