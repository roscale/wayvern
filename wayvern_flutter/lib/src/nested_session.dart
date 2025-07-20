import 'package:flutter/services.dart';

const platform = MethodChannel('wayvern/nested_session');

Future<void> spawnWindow() async {
  final a = await platform.invokeMethod("spawn_window");
  print("Received from native: $a");
}
