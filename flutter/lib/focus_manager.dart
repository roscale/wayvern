import 'package:wayvern_flutter/wayvern_flutter.dart' as wayvern;

// Last in stack is the focused surface.
final List<int> _stack = [];

void initFocusManager() {
  wayvern.events.listen((event) async {
    switch (event) {
      case wayvern.WindowCreated(:final surfaceId):
        await wayvern.focusSurface(surfaceId);
        _stack.add(surfaceId);

      case wayvern.WindowDestroyed(:final surfaceId):
        final inFocus = _stack.last == surfaceId;
        _stack.remove(surfaceId);

        if (inFocus && _stack.isNotEmpty) {
          await wayvern.focusSurface(_stack.last);
        }

      default:
    }
  });
}

void setFocus(int surfaceId) {
  _stack.remove(surfaceId);
  _stack.add(surfaceId);
}
