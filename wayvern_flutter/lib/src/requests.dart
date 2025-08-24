import 'package:wayvern_flutter/src/wayvern_flutter.dart';

Future<void> setInitialWindowSize(int width, int height) async {
  await channel.invokeMethod("setInitialWindowSize", [width, height]);
}

Future<void> resizeWindow(int surfaceId, int width, int height) async {
  await channel.invokeMethod("resizeWindow", [surfaceId, width, height]);
}

Future<void> pointerMotion(int surfaceId, double x, double y) async {
  await channel.invokeMethod("motion", [surfaceId, x, y]);
}

Future<void> pointerButton(int button, bool pressed) async {
  await channel.invokeMethod("button", [button, pressed]);
}

Future<void> focusSurface(int surfaceId, bool focus) async {
  await channel.invokeMethod("focusSurface", [surfaceId, focus]);
}
