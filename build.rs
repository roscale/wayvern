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

const REPO: &str = "https://github.com/meta-flutter/flutter-engine/releases/download";
const LIB_DIR: &str = "target/wayvern/lib";
const FLUTTER_SDK_DIR: &str = "target/wayvern/flutter";

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = std::env::var("PROFILE").unwrap();
    let arch = get_arch();
    let engine_hash = get_engine_hash();

    println!("cargo::rerun-if-changed={LIB_DIR}");

    if should_download_library(&profile, &engine_hash) {
        let archive = download_archive(&profile, arch, &engine_hash);
        let files =
            extract_files_from_tar(&archive, &["libflutter_engine.so", "flutter_embedder.h"]);

        write_files_to_disk(&files, &profile, &engine_hash);
    }

    generate_bindings(&profile, &out_dir);
    link_library(&profile);
}

fn get_arch() -> &'static str {
    match ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        arch => panic!("Architecture {arch} not supported"),
    }
}

fn get_engine_hash() -> String {
    let flutter_exe = which_flutter();
    let bin = flutter_exe.parent().unwrap();
    let engine_version_path = bin.join("internal").join("engine.version");
    let engine_version_path_str = engine_version_path.to_string_lossy();

    println!("cargo::rerun-if-changed={}", &engine_version_path_str);

    read_to_string(&engine_version_path)
        .unwrap_or_else(|_| panic!("Can't read {engine_version_path_str}"))
}

fn which_flutter() -> PathBuf {
    println!("cargo::rerun-if-env-changed=PATH");

    match Command::new("which").arg("flutter").output() {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout);
            PathBuf::from(&*path)
        }
        _ => {
            panic!("Flutter is required but not found in PATH");
        }
    }
}

fn should_download_library(profile: &str, engine_hash: &str) -> bool {
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

    let archive_url = format!("{REPO}/{filename}/{archive_filename}");
    let checksum_url = format!("{REPO}/{filename}/{checksum_filename}");

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

fn link_library(profile: &str) {
    let dir = std::path::absolute(format!("{LIB_DIR}/{profile}")).unwrap();
    let dir = dir.to_str().unwrap();

    println!("cargo:rustc-link-search={dir}");
    println!("cargo:rustc-link-lib=flutter_engine");

    println!("cargo:rustc-link-arg=-Wl,-rpath=$ORIGIN,-rpath={dir}");
}

fn generate_bindings(profile: &str, out_dir: &str) {
    let header = format!("lib/{profile}/flutter_embedder.h");

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
