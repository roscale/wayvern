run profile="debug": (_cargo profile "run")

build profile="debug": (_cargo profile "build")

clean:
  cargo clean
  -rm -r lib

_cargo profile args: (download_flutter_engine profile)
  FLUTTER_ENGINE_HEADER={{absolute_path("lib/" + profile + "/flutter_embedder.h")}} \
  FLUTTER_ENGINE_LIB={{absolute_path("lib/" + profile + "/libflutter_engine.so")}} \
  cargo {{args}}

download_flutter_engine profile:
  ./download_flutter_engine.py {{profile}}

