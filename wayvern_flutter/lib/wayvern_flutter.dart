import 'package:wayvern_flutter/src/window.dart';

export 'package:wayvern_flutter/src/window.dart'
    show setInitialWindowSize, windows, windowCreated, windowDestroyed, motion;

void init() {
  initWindowStreams();
}
