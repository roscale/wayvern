import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

const windowChannel = MethodChannel('wayvern/window');

final _windowCreatedStreamController = StreamController<int>.broadcast();
final _windowDestroyedStreamController = StreamController<int>.broadcast();
final _textureAttachedController = StreamController<(int, int)>.broadcast();

Stream<int> get windowCreated => _windowCreatedStreamController.stream;
Stream<int> get windowDestroyed => _windowDestroyedStreamController.stream;
Stream<(int, int)> get textureAttached => _textureAttachedController.stream;

final windows = Windows();

class Windows with ChangeNotifier {
  final Map<int, Window> map = {};
  final List<Window> list = [];

  void add(int id) {
    final window = Window(id);
    map[id] = window;
    list.add(window);
    notifyListeners();
  }

  void remove(int id) {
    map.remove(id);
    list.removeWhere((window) => window.id == id);
    notifyListeners();
  }
}

class Window with ChangeNotifier {
  final int id;

  int _textureId = 0;
  int _width = 0;
  int _height = 0;

  Window(this.id);

  int get textureId => _textureId;
  int get width => _width;
  int get height => _height;

  void attachTexture(int textureId, int width, int height) {
    _textureId = textureId;
    _width = width;
    _height = height;

    notifyListeners();
  }
}

void initWindowStreams() {
  windowChannel.setMethodCallHandler((call) async {
    if (call.method == "windowCreated") {
      final id = call.arguments as int;

      _windowCreatedStreamController.sink.add(id);
      windows.add(id);
    } else if (call.method == "windowDestroyed") {
      final id = call.arguments as int;

      _windowDestroyedStreamController.sink.add(id);
      windows.remove(id);
    } else if (call.method == "textureAttached") {
      final args = call.arguments as List<int>;

      final surfaceId = args[0];
      final textureId = args[1];
      final width = args[2];
      final height = args[3];

      final window = windows.map[surfaceId];

      if (window == null) {
        print(
          "Attached texture $textureId to a non-existent surface $surfaceId",
        );
        return;
      }

      _textureAttachedController.sink.add((surfaceId, textureId));
      window.attachTexture(textureId, width, height);
    }
  });
}

Future<void> setInitialWindowSize(int width, int height) async {
  await windowChannel.invokeMethod("setInitialWindowSize", [width, height]);
}

Future<void> pointerMotion(int surfaceId, double x, double y) async {
  await windowChannel.invokeMethod("motion", [surfaceId, x, y]);
}

Future<void> pointerButton(int button, bool pressed) async {
  await windowChannel.invokeMethod("button", [button, pressed]);
}
