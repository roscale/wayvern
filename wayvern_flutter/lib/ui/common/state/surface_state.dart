import 'package:fast_immutable_collections/fast_immutable_collections.dart';
import 'package:flutter/material.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:zenith/platform_api.dart';

part '../../../generated/ui/common/state/surface_state.freezed.dart';

@freezed
class SurfaceState with _$SurfaceState {
  const factory SurfaceState({
    required bool soonDestroyed,
    required SurfaceRole role,
    required int viewId,
    required TextureId textureId,
    required TextureId oldTextureId,
    required Offset surfacePosition,
    required Size surfaceSize,
    required double scale,
    required GlobalKey textureKey,
    required IList<int> subsurfacesBelow,
    required IList<int> subsurfacesAbove,
    required Rect inputRegion,
  }) = _SurfaceState;
}

// @Riverpod(keepAlive: true)
// class SurfaceStates extends _$SurfaceStates {
//   @override
//   SurfaceState build(int viewId) {
//     return SurfaceState(
//       role: SurfaceRole.none,
//       viewId: viewId,
//       textureId: TextureId(-1),
//       oldTextureId: TextureId(-1),
//       surfacePosition: Offset.zero,
//       surfaceSize: Size.zero,
//       scale: 1,
//       widgetKey: GlobalKey(),
//       textureKey: GlobalKey(),
//       subsurfacesBelow: [],
//       subsurfacesAbove: [],
//       inputRegion: Rect.zero,
//     );
//   }
//
//   void init() {
//     state = state.copyWith(viewId: viewId);
//   }
//
//   void commit({
//     required SurfaceRole role,
//     required TextureId textureId,
//     required Offset surfacePosition,
//     required Size surfaceSize,
//     required double scale,
//     required List<int> subsurfacesBelow,
//     required List<int> subsurfacesAbove,
//     required Rect inputRegion,
//   }) {
//     final platform = ref.read(platformApiProvider.notifier);
//
//     assert(textureId != state.oldTextureId);
//
//     TextureId oldTexture = state.oldTextureId;
//     TextureId currentTexture = state.textureId;
//
//     if (textureId != currentTexture) {
//       if (oldTexture.value != -1) {
//         platform.textureFinalizer.detach(oldTexture);
//       }
//       oldTexture = currentTexture;
//       currentTexture = textureId;
//     }
//
//     state = state.copyWith(
//       role: role,
//       textureId: currentTexture,
//       oldTextureId: oldTexture,
//       surfacePosition: surfacePosition,
//       surfaceSize: surfaceSize,
//       scale: scale,
//       subsurfacesBelow: subsurfacesBelow,
//       subsurfacesAbove: subsurfacesAbove,
//       inputRegion: inputRegion,
//     );
//   }
//
//   void unmap() {
//     state = state.copyWith(
//       role: SurfaceRole.none,
//     );
//   }
//
//   void dispose() {
//     // Cascading dispose of all surface roles.
//     switch (state.role) {
//       case SurfaceRole.xdgSurface:
//         ref.read(xdgSurfaceStatesProvider(viewId).notifier).dispose();
//         break;
//       case SurfaceRole.subsurface:
//         ref.read(subsurfaceStatesProvider(viewId).notifier).dispose();
//         break;
//       case SurfaceRole.none:
//         break;
//     }
//
//     ref.invalidate(surfaceWidgetProvider(viewId));
//
//     // This refresh seems very redundant but it's actually needed.
//     // Without refresh, the state persists in memory and if a Finalizer attaches to an object
//     // inside the state, it will never call its finalization callback.
//     final _ = ref.refresh(surfaceStatesProvider(viewId));
//     ref.invalidate(surfaceStatesProvider(viewId));
//   }
// }

// Order is very important here when decoding from index.
enum SurfaceRole {
  none,
  xdgSurface,
  subsurface,
}
