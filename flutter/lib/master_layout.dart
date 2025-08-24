import 'dart:async';

import 'package:flutter/material.dart';
import 'package:wayvern_flutter/wayvern_flutter.dart' as wayvern;

class InteractiveWindow extends StatelessWidget {
  final wayvern.Window window;

  const InteractiveWindow({required this.window, super.key});

  @override
  Widget build(BuildContext context) {
    return wayvern.SurfaceInputListener(
      surfaceId: window.id,
      focusNode: FocusNode(),
      child: Texture(textureId: window.texture?.textureId ?? 0),
    );
  }
}

class MasterLayout extends StatefulWidget {
  const MasterLayout({super.key});

  @override
  State<MasterLayout> createState() => _MasterLayoutState();
}

class _MasterLayoutState extends State<MasterLayout> {
  late StreamSubscription subscription;
  final List<int> windows = [];

  @override
  void initState() {
    super.initState();

    subscription = wayvern.events.listen((event) {
      switch (event) {
        case wayvern.WindowCreated(:final surfaceId):
          setState(() {
            windows.add(surfaceId);
          });

        case wayvern.WindowDestroyed(:final surfaceId):
          setState(() {
            windows.remove(surfaceId);
          });

        case wayvern.TextureAttached():
          setState(() {});
      }
    });
  }

  @override
  void dispose() {
    subscription.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final width = constraints.biggest.width.toInt();
        final height = constraints.biggest.height.toInt();

        if (windows.isEmpty) {
          wayvern.setInitialWindowSize(width, height);
          return SizedBox();
        }

        wayvern.resizeWindow(
          windows[0],
          windows.length == 1 ? width : width ~/ 2,
          height,
        );

        wayvern.setInitialWindowSize(width ~/ 2, height ~/ windows.length);

        final master = wayvern.windows.map[windows[0]]!;

        if (windows.length == 1) {
          wayvern.resizeWindow(windows[0], width, height);
          wayvern.setInitialWindowSize(width ~/ 2, height);

          return SizedBox(
            width: width.toDouble(),
            height: height.toDouble(),
            child: InteractiveWindow(window: master),
          );
        }

        wayvern.resizeWindow(windows[0], width ~/ 2, height);
        wayvern.setInitialWindowSize(width ~/ 2, height ~/ windows.length);

        for (final slave in windows.skip(1)) {
          wayvern.resizeWindow(
            slave,
            width ~/ 2,
            height ~/ (windows.length - 1),
          );
        }

        final slaves =
            windows.skip(1).map((id) => wayvern.windows.map[id]!).toList();

        final slaveWidgets =
            slaves.map((slave) {
              return Expanded(child: InteractiveWindow(window: slave));
            }).toList();

        return Row(
          children: [
            Expanded(child: InteractiveWindow(window: master)),
            Expanded(child: Column(children: slaveWidgets)),
          ],
        );
      },
    );
  }
}
