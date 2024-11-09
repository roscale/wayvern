import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:zenith/ui/common/state/wayland_state.dart';
import 'package:zenith/ui/common/subsurface.dart';
import 'package:zenith/ui/common/surface_size.dart';
import 'package:zenith/ui/common/view_input_listener.dart';

class Surface extends ConsumerWidget {
  final int viewId;

  const Surface({
    super.key,
    required this.viewId,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return SurfaceSize(
      viewId: viewId,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          _Subsurfaces(
            viewId: viewId,
            layer: _SubsurfaceLayer.below,
          ),
          ViewInputListener(
            viewId: viewId,
            child: Consumer(
              builder: (BuildContext context, WidgetRef ref, Widget? child) {
                Key key = ref.watch(waylandProviderProvider
                    .select((v) => v.surfaces[viewId]!.textureKey));
                int textureId = ref.watch(waylandProviderProvider
                    .select((v) => v.surfaces[viewId]!.textureId.value));

                return Texture(
                  key: key,
                  textureId: textureId,
                );
              },
            ),
          ),
          _Subsurfaces(
            viewId: viewId,
            layer: _SubsurfaceLayer.above,
          ),
        ],
      ),
    );
  }
}

class _Subsurfaces extends ConsumerWidget {
  final int viewId;
  final _SubsurfaceLayer layer;

  const _Subsurfaces({
    required this.viewId,
    required this.layer,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selector = layer == _SubsurfaceLayer.below
        ? (WaylandStates s) => s.surfaces[viewId]!.subsurfacesBelow
        : (WaylandStates s) => s.surfaces[viewId]!.subsurfacesAbove;

    List<Widget> subsurfaces = ref
        .watch(waylandProviderProvider.select(selector))
        .where(
          (id) => ref.watch(
            waylandProviderProvider.select(
              // Surface might not exist when its destruction notifies this callback.
              (s) => s.surfaceExists(id) && s.isMapped(id),
            ),
          ),
        )
        .map(
          (id) => Subsurface(
            key: ValueKey(id),
            viewId: id,
          ),
        )
        .toList();

    return Stack(
      clipBehavior: Clip.none,
      children: subsurfaces,
    );
  }
}

enum _SubsurfaceLayer { below, above }
