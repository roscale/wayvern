import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zenith/ui/desktop/state/window_move_provider.dart';
import 'package:zenith/ui/desktop/state/window_position_provider.dart';

class MoveWindowOnPanGestureDetector extends ConsumerWidget {
  final bool enabled;
  final int viewId;
  final Widget child;

  const MoveWindowOnPanGestureDetector({
    super.key,
    this.enabled = true,
    required this.viewId,
    required this.child,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return GestureDetector(
      onPanDown: enabled
          ? (DragDownDetails details) {
              Offset startPosition = ref.read(windowPositionProvider(viewId));
              ref
                  .read(windowMoveProvider(viewId).notifier)
                  .startMove(startPosition);
            }
          : null,
      onPanUpdate: enabled
          ? (DragUpdateDetails details) {
              ref.read(windowMoveProvider(viewId).notifier).move(details.delta);
            }
          : null,
      onPanEnd: enabled
          ? (_) {
              ref.read(windowMoveProvider(viewId).notifier).endMove();
            }
          : null,
      onPanCancel: enabled
          ? () {
              ref.read(windowMoveProvider(viewId).notifier).cancelMove();
            }
          : null,
      child: child,
    );
  }
}
