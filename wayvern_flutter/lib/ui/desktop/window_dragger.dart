import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:zenith/ui/desktop/move_window_on_pan_gesture_detector.dart';

class WindowDragger extends StatefulWidget {
  final int viewId;
  final Widget child;

  const WindowDragger({
    super.key,
    required this.viewId,
    required this.child,
  });

  @override
  State<WindowDragger> createState() => _WindowDraggerState();
}

class _WindowDraggerState extends State<WindowDragger> {
  final focusNode = FocusNode();
  bool dragging = false;

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: focusNode,
      onKeyEvent: (_) {
        setState(() {
          dragging = HardwareKeyboard.instance.isMetaPressed;
        });
      },
      child: MoveWindowOnPanGestureDetector(
        enabled: dragging,
        viewId: widget.viewId,
        child: AbsorbPointer(
          absorbing: dragging,
          child: widget.child,
        ),
      ),
    );
  }

  @override
  void dispose() {
    focusNode.dispose();
    super.dispose();
  }
}
