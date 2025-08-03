import 'package:flutter/services.dart';

const windowChannel = MethodChannel('wayvern/window');

const StreamController<int>

Future<void> setInitialWindowSize(int width, int height) async {
  await windowChannel.invokeMethod("setInitialWindowSize", [width, height]);
}

