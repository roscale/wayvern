import 'package:flutter/services.dart';

const platform = MethodChannel('wayvern/window');

Future<void> setInitialWindowSize(int width, int height) async {
  await platform.invokeMethod("setInitialWindowSize", [width, height]);
}
