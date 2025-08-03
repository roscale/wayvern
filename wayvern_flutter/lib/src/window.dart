import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

const windowChannel = MethodChannel('wayvern/window');

final windowCreatedStreamController = StreamController<int>.broadcast();
final windowDestroyedStreamController = StreamController<int>.broadcast();

Stream<int> get windowCreated => windowCreatedStreamController.stream;
Stream<int> get windowDestroyed => windowDestroyedStreamController.stream;
final windows = Windows();

class Windows with ChangeNotifier {
  final ids = [];

  void add(int id) {
    ids.add(id);
    notifyListeners();
  }

  void remove(int id) {
    ids.remove(id);
    notifyListeners();
  }
}

void initWindowStreams() {
  windowChannel.setMethodCallHandler((call) async {
    if (call.method == "windowCreated") {
      final id = call.arguments as int;

      windowCreatedStreamController.sink.add(id);
      windows.add(id);
    } else if (call.method == "windowDestroyed") {
      final id = call.arguments as int;

      windowDestroyedStreamController.sink.add(id);
      windows.remove(id);
    }
  });
}

Future<void> setInitialWindowSize(int width, int height) async {
  await windowChannel.invokeMethod("setInitialWindowSize", [width, height]);
}
