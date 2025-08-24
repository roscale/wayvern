import 'package:flutter/services.dart';
import 'package:wayvern_flutter/src/events.dart';
import 'package:wayvern_flutter/src/texture.dart';

const channel = MethodChannel('wayvern/window');

void init() {
  channel.setMethodCallHandler((call) async {
    if (call.method == "windowCreated") {
      handleWindowCreated(call.arguments);
    } else if (call.method == "windowDestroyed") {
      handleWindowDestroyed(call.arguments);
    } else if (call.method == "textureAttached") {
      handleTextureAttached(call.arguments);
    } else {
      throw MissingPluginException("Method ${call.method} is not implemented");
    }
  });
}

void handleWindowCreated(dynamic arguments) {
  final id = arguments as int;
  eventsSink.add(WindowCreated(id));
}

void handleWindowDestroyed(dynamic arguments) {
  final id = arguments as int;
  eventsSink.add(WindowDestroyed(id));
}

void handleTextureAttached(dynamic arguments) {
  final args = arguments as List<int>;

  final surfaceId = args[0];
  final textureId = args[1];
  final width = args[2];
  final height = args[3];

  final texture = Texture(textureId: textureId, width: width, height: height);

  eventsSink.add(TextureAttached(surfaceId, texture));
}
