use std::env;
use std::str::FromStr;

fn main() {
    link_libflutter_engine();
}

fn link_libflutter_engine() {
    link_libgl();

    let flutter_engine_config = match env::var("FLUTTER_ENGINE") {
        Ok(config) => FlutterEngineConfig::from_str(&config).unwrap_or(FlutterEngineConfig::Debug),
        Err(_) => FlutterEngineConfig::Debug,
    };

    let libflutter_engine_dirs = "third_party/flutter_engine";

    let libflutter_engine_dir = match flutter_engine_config {
        FlutterEngineConfig::Debug => format!("{libflutter_engine_dirs}/debug"),
        FlutterEngineConfig::Profile => format!("{libflutter_engine_dirs}/profile"),
        FlutterEngineConfig::Release => format!("{libflutter_engine_dirs}/release"),
    };

    println!("cargo:rerun-if-changed={libflutter_engine_dir}");

    println!("cargo:rustc-link-search={libflutter_engine_dir}");
    println!("cargo:rustc-link-lib=flutter_engine");

    let rpath = if option_env!("BUNDLE").is_some() {
        "$ORIGIN/lib"
    } else {
        &libflutter_engine_dir
    };

    println!("cargo:rustc-link-arg=-Wl,-rpath={rpath}");
}

fn link_libgl() {
    // libflutter_engine.so uses libGL.so, not the Rust code.
    // rustc has no idea and thinks libGL.so is not needed.
    // --no-as-needed is needed to force the linker to link libGL.so.
    // We manually put -lGL here because `println!("cargo:rustc-link-lib=GL")` doesn't work.
    println!("cargo:rustc-link-arg=-Wl,--no-as-needed,-lGL");
}

enum FlutterEngineConfig {
    Debug,
    Profile,
    Release,
}

impl FromStr for FlutterEngineConfig {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(FlutterEngineConfig::Debug),
            "profile" => Ok(FlutterEngineConfig::Profile),
            "release" => Ok(FlutterEngineConfig::Release),
            _ => Err(()),
        }
    }
}
