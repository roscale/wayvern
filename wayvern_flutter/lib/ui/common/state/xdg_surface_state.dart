import 'package:fast_immutable_collections/fast_immutable_collections.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part '../../../generated/ui/common/state/xdg_surface_state.freezed.dart';

@freezed
class XdgSurfaceState with _$XdgSurfaceState {
  const factory XdgSurfaceState({
    required XdgSurfaceRole role,
    required Rect visibleBounds,
    required IList<int> popups,
  }) = _XdgSurfaceState;
}

enum XdgSurfaceRole {
  none,
  toplevel,
  popup,
}
