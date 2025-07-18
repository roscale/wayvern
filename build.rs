use std::{
    collections::HashMap,
    env::{self, consts::ARCH},
    fs::{self, File, read_to_string},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use flate2::bufread::GzDecoder;
use sha2::{Digest, Sha256};
use tar::Archive;

const FLUTTER_REPO: &str = "https://github.com/flutter/flutter";
const ENGINE_REPO: &str = "https://github.com/meta-flutter/flutter-engine/releases/download";

const FLUTTER_SDK_DIR: &str = "target/wayvern/flutter";
const LIB_DIR: &str = "target/wayvern/lib";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = std::env::var("PROFILE").unwrap();
    let flutter_version = env!("FLUTTER_VERSION");

    println!("cargo::rerun-if-changed=target/wayvern/flutter/version");
    println!("cargo::rerun-if-changed=target/wayvern/lib");
    println!("cargo::rerun-if-changed=flutter/lib");

    if Path::new(FLUTTER_SDK_DIR).exists() {
        if should_update_flutter_sdk(flutter_version) {
            update_flutter_sdk(flutter_version);
        }
    } else {
        clone_flutter_sdk(flutter_version)
    }

    let arch = get_arch();
    let engine_hash = get_engine_hash();

    if should_download_engine(&profile, &engine_hash) {
        let archive = download_archive(&profile, arch, &engine_hash);
        let files =
            extract_files_from_tar(&archive, &["libflutter_engine.so", "flutter_embedder.h"]);

        write_files_to_disk(&files, &profile, &engine_hash);
    }

    generate_bindings(&profile, &out_dir);
    link_library(&profile);

    flutter_build(&profile);
}

fn should_update_flutter_sdk(version: &str) -> bool {
    let version_on_disk = run_command_get_stdout(
        Command::new("git")
            .current_dir(FLUTTER_SDK_DIR)
            .arg("describe")
            .arg("--exact-match")
            .arg("--tags"),
    );
    let version_on_disk = version_on_disk.trim();

    version_on_disk != version
}

fn update_flutter_sdk(version: &str) {
    run_command(
        Command::new("git")
            .current_dir(FLUTTER_SDK_DIR)
            .arg("checkout")
            .arg(version),
    );
}

fn clone_flutter_sdk(version: &str) {
    run_command(
        Command::new("git")
            .arg("clone")
            .arg("--single-branch")
            .arg("--branch")
            .arg(version)
            .arg(FLUTTER_REPO)
            .arg(FLUTTER_SDK_DIR),
    );
}

fn run_command(command: &mut Command) {
    command
        .spawn()
        .unwrap_or_else(|err| panic!("Failed to spawn command: {err}"))
        .wait()
        .unwrap_or_else(|err| panic!("Failed to wait command: {err}"));
}

fn run_command_get_stdout(command: &mut Command) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("Failed to spawn command: {err}"));

    if !output.status.success() {
        panic!("Command failed, {:?}", output.status);
    }

    String::from_utf8(output.stdout).expect("Command output is not UTF-8")
}

fn get_arch() -> &'static str {
    match ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        arch => panic!("Architecture {arch} not supported"),
    }
}

fn get_engine_hash() -> String {
    let path = format!("{FLUTTER_SDK_DIR}/bin/internal/engine.version");

    println!("cargo::rerun-if-changed={}", &path);

    read_to_string(&path).unwrap_or_else(|err| panic!("Failed to read {path}, {err}"))
}

fn should_download_engine(profile: &str, engine_hash: &str) -> bool {
    let engine_hash_file = format!("{LIB_DIR}/{profile}/engine_hash");

    let hash_on_disk = read_to_string(engine_hash_file);
    hash_on_disk
        .map(|hash_on_disk| engine_hash != hash_on_disk)
        .unwrap_or(true)
}

fn download_archive(profile: &str, arch: &str, engine_hash: &str) -> Vec<u8> {
    let filename = format!("linux-engine-sdk-{profile}-{arch}-{engine_hash}");
    let archive_filename = format!("{filename}.tar.gz");
    let checksum_filename = format!("{archive_filename}.sha256");

    let archive_url = format!("{ENGINE_REPO}/{filename}/{archive_filename}");
    let checksum_url = format!("{ENGINE_REPO}/{filename}/{checksum_filename}");

    let archive = download_binary(&archive_url);

    let checksum = download_text(&checksum_url);
    let checksum = checksum
        .split_once(" ")
        .unwrap_or_else(|| panic!("Failed to read checksum"))
        .0;

    verify_sha256(&archive, checksum);

    archive
}

fn download_binary(url: &str) -> Vec<u8> {
    reqwest::blocking::get(url)
        .unwrap_or_else(|_| panic!("Failed to download {url}"))
        .error_for_status()
        .unwrap_or_else(|_| panic!("Failed to download {url}"))
        .bytes()
        .unwrap_or_else(|_| panic!("Failed to download {url}"))
        .to_vec()
}

fn download_text(url: &str) -> String {
    reqwest::blocking::get(url)
        .unwrap_or_else(|_| panic!("Failed to download {url}"))
        .error_for_status()
        .unwrap_or_else(|_| panic!("Failed to download {url}"))
        .text()
        .unwrap_or_else(|_| panic!("Failed to download {url}"))
}

fn verify_sha256(archive: &[u8], checksum: &str) {
    let digest = Sha256::digest(archive);
    assert_eq!(hex::encode(digest), checksum, "Checksum failed");
}

fn extract_files_from_tar(blob: &[u8], filenames: &[&str]) -> HashMap<String, Vec<u8>> {
    let mut found = HashMap::new();
    let mut archive = Archive::new(GzDecoder::new(blob));

    for entry in archive.entries().expect("Failed to read tar entries") {
        let mut entry = entry.expect("Failed to read a tar entry");

        let path = entry.path().expect("Failed to get entry path").into_owned();

        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();

            if filenames.contains(&name.as_ref()) {
                let mut contents = Vec::new();

                entry
                    .read_to_end(&mut contents)
                    .expect("Failed to read file contents");

                found.insert(name.into_owned(), contents);
            }
        }
    }

    let missing: Vec<_> = filenames
        .iter()
        .copied()
        .filter(|name| !found.contains_key(*name))
        .collect();

    if !missing.is_empty() {
        panic!("Missing from archive: {}", missing.join(", "));
    }

    found
}

fn write_files_to_disk(files: &HashMap<String, Vec<u8>>, profile: &str, engine_hash: &str) {
    let lib_dir = Path::new(LIB_DIR).join(profile);

    fs::create_dir_all(&lib_dir).expect("Failed to create lib directory");

    for (name, content) in files {
        let file_path = lib_dir.join(name);
        let mut file = File::create(&file_path).expect("Failed to create output file");

        file.write_all(content)
            .expect("Failed to write file content");
    }

    let mut hash_file =
        File::create(lib_dir.join("engine_hash")).expect("Failed to create engine_hash file");

    hash_file
        .write_all(engine_hash.as_bytes())
        .expect("Failed to write engine_hash");
}

fn generate_bindings(profile: &str, out_dir: &str) {
    let header = format!("{LIB_DIR}/{profile}/flutter_embedder.h");

    let bindings = bindgen::Builder::default()
        .header(header)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_dir = PathBuf::from(out_dir);

    bindings
        .write_to_file(out_dir.join("flutter_engine_sys.rs"))
        .expect("Couldn't write bindings");
}

fn link_library(profile: &str) {
    let dir = std::path::absolute(format!("{LIB_DIR}/{profile}")).unwrap();
    let dir = dir.to_str().unwrap();

    println!("cargo:rustc-link-search={dir}");
    println!("cargo:rustc-link-lib=flutter_engine");

    println!("cargo:rustc-link-arg=-Wl,-rpath=$ORIGIN,-rpath={dir}");
}

fn flutter_build(profile: &str) {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    run_command(
        Command::new(format!("{manifest_dir}/target/wayvern/flutter/bin/flutter"))
            .current_dir("flutter")
            .arg("build")
            .arg("linux")
            .arg(format!("--{profile}")),
    );
}
