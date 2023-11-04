import 'package:flutter/material.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:zenith/ui/common/state/surface_state.dart';
import 'package:zenith/ui/common/state/xdg_surface_state.dart';
import 'package:zenith/ui/common/subsurface.dart';

part '../../../generated/ui/common/state/subsurface_state.freezed.dart';

part '../../../generated/ui/common/state/subsurface_state.g.dart';

@Riverpod(keepAlive: true)
Subsurface subsurfaceWidget(SubsurfaceWidgetRef ref, int viewId) {
  return Subsurface(
    key: ref.watch(subsurfaceStatesProvider(viewId).select((state) => state.widgetKey)),
    viewId: viewId,
  );
}

@freezed
class SubsurfaceState with _$SubsurfaceState {
  const factory SubsurfaceState({
    required bool mapped,
    required int parent,
    required Offset position, // relative to the parent
    required Key widgetKey,
  }) = _SubsurfaceState;
}

@Riverpod(keepAlive: true)
class SubsurfaceStates extends _$SubsurfaceStates {
  @override
  SubsurfaceState build(int viewId) {
    ref.listen(surfaceStatesProvider(viewId).select((state) => state.textureId), (_, __) => _checkIfMapped());

    return SubsurfaceState(
      mapped: false,
      parent: 0,
      position: Offset.zero,
      widgetKey: GlobalKey(),
    );
  }

  void _checkIfMapped() {
    bool mapped = ref.read(surfaceStatesProvider(viewId)).textureId.value != -1 &&
        ref.read(xdgSurfaceStatesProvider(state.parent)).mapped;

    state = state.copyWith(
      mapped: mapped,
    );
  }

  void commit({required int parent, required Offset position}) {
    ref.listen(xdgSurfaceStatesProvider(parent).select((state) => state.mapped), (_, __) => _checkIfMapped());

    state = state.copyWith(
      parent: parent,
      position: position,
    );
  }

  void dispose() {
    ref.invalidate(subsurfaceWidgetProvider(viewId));
    ref.invalidate(subsurfaceStatesProvider(viewId));
  }
}
