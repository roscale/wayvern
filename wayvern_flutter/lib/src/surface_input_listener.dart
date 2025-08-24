import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:wayvern_flutter/src/mouse_button_tracker.dart';
import 'package:wayvern_flutter/src/requests.dart';

final _mouseButtonTracker = MouseButtonTracker();

class SurfaceInputListener extends StatelessWidget {
  final int surfaceId;
  final FocusNode focusNode;
  final Widget child;

  const SurfaceInputListener({
    super.key,
    required this.surfaceId,
    required this.focusNode,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerDown: (event) {
        focusNode.requestFocus();
        sendButtons(event);
      },
      onPointerMove: (event) {
        sendMotion(event);
        sendButtons(event);
      },
      onPointerUp: sendButtons,
      onPointerHover: sendMotion,
      onPointerCancel: sendButtons,
      child: Focus(
        focusNode: focusNode,
        onFocusChange: (focused) => focusSurface(surfaceId, focused),
        child: AbsorbPointer(child: child),
      ),
    );
  }

  Future<void> sendMotion(PointerEvent event) async {
    await pointerMotion(
      surfaceId,
      event.localPosition.dx,
      event.localPosition.dy,
    );
  }

  Future<void> sendButtons(PointerEvent event) async {
    final buttons = _mouseButtonTracker.trackButtons(
      event.device,
      event.buttons,
    );

    for (final button in buttons) {
      await pointerButton(button.button, button.pressed);
    }
  }
}
