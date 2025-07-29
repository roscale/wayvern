[working-directory: "flutter"]
flutter *args:
  ../target/wayvern/flutter/bin/flutter {{args}}

build mode="":
  cargo build {{ mode }}
  just flutter build linux {{ if mode == "" { "--debug" } else { mode } }}

run mode="": (build mode)
  cargo run {{ mode }}

clean:
  cargo clean
  just flutter clean

