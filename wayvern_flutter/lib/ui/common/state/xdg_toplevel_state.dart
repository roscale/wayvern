import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:zenith/platform_api.dart';
import 'package:zenith/ui/common/state/wayland_state.dart';
import 'package:zenith/ui/common/state/xdg_surface_state.dart';
import 'package:zenith/ui/common/xdg_toplevel_surface.dart';

part '../../../generated/ui/common/state/xdg_toplevel_state.freezed.dart';


@freezed
class XdgToplevelState with _$XdgToplevelState {
  const factory XdgToplevelState({
    required bool visible,
    required Key virtualKeyboardKey,
    required FocusNode focusNode,
    required Object interactiveMoveRequested,
    required ResizeEdgeObject interactiveResizeRequested,
    required ToplevelDecoration? decoration,
    required String? title,
    required String? appId,
    required Tiling? tilingRequested,
  }) = _XdgToplevelState;
}

enum ResizeEdge {
  topLeft,
  top,
  topRight,
  right,
  bottomRight,
  bottom,
  bottomLeft,
  left;

  static ResizeEdge fromInt(int n) {
    switch (n) {
      case 1:
        return top;
      case 2:
        return bottom;
      case 4:
        return left;
      case 5:
        return topLeft;
      case 6:
        return bottomLeft;
      case 8:
        return right;
      case 9:
        return topRight;
      case 10:
        return bottomRight;
      default:
        return bottomRight;
    }
  }
}

class ResizeEdgeObject {
  final ResizeEdge edge;

  ResizeEdgeObject(this.edge);
}

enum ToplevelDecoration {
  clientSide,
  serverSide;

  static ToplevelDecoration fromInt(int n) {
    switch (n) {
      case 1:
        return clientSide;
      case 2:
        return serverSide;
      default:
        throw ArgumentError('Invalid ToplevelDecoration value: $n');
    }
  }
}

enum Tiling {
  none,
  maximized,
}
