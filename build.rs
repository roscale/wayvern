use std::path::Path;

fn main() {
    link_library();
}

fn link_library() {
    let lib = env!("FLUTTER_ENGINE_LIB");
    let parent = Path::new(lib).parent().unwrap();
    let parent_str = parent.to_str().unwrap();

    println!("cargo:rustc-link-search={parent_str}");
    println!("cargo:rustc-link-lib=flutter_engine");

    println!("cargo:rustc-link-arg=-Wl,-rpath=$ORIGIN,-rpath={parent_str}");
}
