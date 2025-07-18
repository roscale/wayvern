[working-directory: 'flutter']
flutter *args:
  ../target/wayvern/flutter/bin/flutter {{args}}

clean: (flutter "clean")
  cargo clean

