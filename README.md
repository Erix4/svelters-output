# How to test

To test, you can use the wasm-reloader tool.

Simply run `wasm-reloader` at the root of the project, and ensure:
- Your code is in src, in library format (i.e. `lib.rs` with `#[wasm_bindgen]` functions)
- You have an index.html in root that imports the generated `pkg` and calls the exported functions.

Then, you can edit your Rust code, and see the changes reflected in the browser without needing to refresh.