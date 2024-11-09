use std::env;
use std::path::PathBuf;

fn main() {
    generate_embedder_bindings();
}

fn generate_embedder_bindings() {
    let embedder_header_path = "src/embedder.h";
    println!("cargo:rerun-if-changed={embedder_header_path}");

    let bindings = bindgen::Builder::default()
        .header(embedder_header_path)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("embedder.rs"))
        .expect("Couldn't write bindings!");
}
