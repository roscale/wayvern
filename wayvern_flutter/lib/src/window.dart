import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

const windowChannel = MethodChannel('wayvern/window');

final windowCreatedStreamController = StreamController<int>.broadcast();
final windowDestroyedStreamController = StreamController<int>.broadcast();
final textureAttachedController = StreamController<(int, int)>.broadcast();

Stream<int> get windowCreated => windowCreatedStreamController.stream;
Stream<int> get windowDestroyed => windowDestroyedStreamController.stream;
Stream<(int, int)> get textureAttached => textureAttachedController.stream;

final windows = Windows();

class Windows with ChangeNotifier {
  final Map<int, Window> map = {};
  final order = [];

  void add(int id) {
    map[id] = Window(id);
    order.add(id);
    notifyListeners();
  }

  void remove(int id) {
    map.remove(id);
    order.remove(id);
    notifyListeners();
  }
}

class Window with ChangeNotifier {
  final int id;
  int? _textureId;

  Window(this.id);

  int? get textureId => _textureId;

  set textureId(int? textureId) {
    _textureId = textureId;
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
    } else if (call.method == "textureAttached") {
      final args = call.arguments as List<int>;

      final surfaceId = args[0];
      final textureId = args[1];

      textureAttachedController.sink.add((surfaceId, textureId));

      final window = windows.map[surfaceId];
      if (window == null) {
        print(
          "Attached texture $textureId to a non-existent surface $surfaceId",
        );
        return;
      }

      window.textureId = textureId;
    }
  });
}

Future<void> setInitialWindowSize(int width, int height) async {
  await windowChannel.invokeMethod("setInitialWindowSize", [width, height]);
}
