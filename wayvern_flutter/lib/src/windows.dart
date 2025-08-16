import 'package:flutter/foundation.dart';
import 'package:wayvern_flutter/src/events.dart';

final windows = Windows();

void initWindowTracker() {
  events.listen((event) {
    switch (event) {
      case WindowCreated(:final surfaceId):
        windows.add(surfaceId);

      case WindowDestroyed(:final surfaceId):
        windows.remove(surfaceId);

      case TextureAttached(
        :final surfaceId,
        :final textureId,
        :final width,
        :final height,
      ):
        windows.map[surfaceId]!.attachTexture(textureId, width, height);
    }
  });
}

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
