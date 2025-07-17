#!/usr/bin/env python3

import hashlib
import io
import os
import platform
import subprocess
import sys
import tarfile
import urllib.request

from pathlib import Path


repo = "https://github.com/meta-flutter/flutter-engine/releases/download"


def main():
    if len(sys.argv) != 2:
        print("Usage: download_flutter_engine.py <profile>", file=sys.stderr)
        sys.exit(1)

    profile = sys.argv[1]

    if profile not in ["debug", "release"]:
        print(f"Invalid profile {profile}. Allowed profiles: debug, release.", file=sys.stderr)
        sys.exit(1)

    arch = detect_arch()
    engine_hash = get_engine_hash()

    if not should_download_library(profile, engine_hash):
        return;

    print("Downloading Flutter engine shared library")

    archive = download_archive(profile, arch, engine_hash)
    files = extract_files_from_tar(archive, ["libflutter_engine.so", "flutter_embedder.h"])

    write_files_to_disk(files, profile, engine_hash)


def run_cmd(cmd) -> str:
    try:
        return subprocess.check_output(cmd, shell=True, text=True).strip()

    except subprocess.CalledProcessError as e:
        print(f"Command failed: {cmd}", file=sys.stderr)
        sys.exit(e.returncode)


def detect_arch() -> str:
    machine = platform.machine()
    mapping = {
        "x86_64": "x86_64",
        "aarch64": "arm64",
    }
    arch = mapping.get(machine.lower())
    if not arch:
        print(f"Unsupported architecture: {machine}", file=sys.stderr)
        sys.exit(1)

    return arch


def get_engine_hash() -> str:
    short_engine_hash = run_cmd('flutter --version | sed -n "s/^Engine • revision //p"')

    if not short_engine_hash:
        print("Error: engine hash not found", file=sys.stderr)
        sys.exit(1)

    flutter_exe = run_cmd("which flutter")
    flutter_dir = os.path.dirname(flutter_exe)

    return run_cmd(f'git -C "{flutter_dir}" rev-parse {short_engine_hash}')


def should_download_library(profile: str, engine_hash: str) -> bool:
    try:
        with open(f"lib/{profile}/engine_hash") as f:
            return f.read() != engine_hash

    except OSError:
        return True


def download_text(url: str) -> str:
    with urllib.request.urlopen(url) as response:
        return response.read().decode().strip()


def download_binary(url: str) -> bytes:
    with urllib.request.urlopen(url) as response:
        return response.read()


def verify_sha256(blob: bytes, expected: str):
    actual = hashlib.sha256(blob).hexdigest()

    if actual != expected:
        print(f"Checksum mismatch:\n  Expected: {expected}\n  Actual:   {actual}", file=sys.stderr)
        sys.exit(1)


def download_archive(profile: str, arch: str, engine_hash: str) -> bytes:
    filename = f"linux-engine-sdk-{profile}-{arch}-{engine_hash}"
    archive_filename = f"{filename}.tar.gz"
    checksum_filename = f"{archive_filename}.sha256"

    archive_url = f"{repo}/{filename}/{archive_filename}"
    checksum_url = f"{repo}/{filename}/{checksum_filename}"

    archive = download_binary(archive_url)
    checksum = download_text(checksum_url).split(" ")[0]

    verify_sha256(archive, checksum)

    return archive


def extract_files_from_tar(blob: bytes, filenames: list[str]) -> dict[str, bytes]:
    found = {}

    with tarfile.open(fileobj=io.BytesIO(blob), mode="r:gz") as tar:
        for member in tar.getmembers():
            name = os.path.basename(member.name)
            if name in filenames:
                fileobj = tar.extractfile(member)
                if fileobj:
                    found[name] = fileobj.read()

        missing = set(filenames) - found.keys()
        if missing:
            print(f"Missing from archive: {', '.join(missing)}", file=sys.stderr)
            sys.exit(1)

    return found


def write_files_to_disk(files: dict[str, bytes], profile: str, engine_hash: str):
    lib_dir = Path(f"lib/{profile}")
    lib_dir.mkdir(parents=True, exist_ok=True)

    for name, content in files.items():
        with open(lib_dir / name, "wb") as f:
            f.write(content)

    with open(lib_dir / "engine_hash", "w") as f:
        f.write(engine_hash)


if __name__ == "__main__":
    main()

