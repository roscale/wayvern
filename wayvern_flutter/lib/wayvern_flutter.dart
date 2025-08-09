import 'package:wayvern_flutter/src/window.dart';

export 'package:wayvern_flutter/src/window.dart'
    show setInitialWindowSize, windows, windowCreated, windowDestroyed;

export 'package:wayvern_flutter/src/surface_input_listener.dart'
    show SurfaceInputListener;

void init() {
  initWindowStreams();
}
