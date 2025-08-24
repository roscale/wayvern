import 'package:flutter/foundation.dart';
import 'package:wayvern_flutter/src/events.dart';
import 'package:wayvern_flutter/src/texture.dart';

final windows = Windows();

void initWindowTracker() {
  events.listen((event) {
    switch (event) {
      case WindowCreated(:final surfaceId):
        windows.add(surfaceId);

      case WindowDestroyed(:final surfaceId):
        windows.remove(surfaceId);

      case TextureAttached(:final surfaceId, :final texture):
        windows.map[surfaceId]!.attachTexture(texture);
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

  Texture? _texture;

  Window(this.id);

  Texture? get texture => _texture;

  void attachTexture(Texture texture) {
    _texture = texture;
    notifyListeners();
  }
}
