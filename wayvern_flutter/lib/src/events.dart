import 'dart:async';

import 'package:wayvern_flutter/src/texture.dart';

final _streamController = StreamController<Event>.broadcast();

final eventsSink = _streamController.sink;
final events = _streamController.stream;

sealed class Event {}

final class WindowCreated extends Event {
  final int surfaceId;

  WindowCreated(this.surfaceId);
}

final class WindowDestroyed extends Event {
  final int surfaceId;

  WindowDestroyed(this.surfaceId);
}

final class TextureAttached extends Event {
  final int surfaceId;
  final Texture texture;

  TextureAttached(this.surfaceId, this.texture);
}
