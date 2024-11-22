import 'package:fast_immutable_collections/fast_immutable_collections.dart';
import 'package:flutter/material.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:zenith/platform_api.dart';
import 'package:zenith/ui/common/popup.dart';
import 'package:zenith/ui/common/popup_stack.dart';
import 'package:zenith/ui/common/state/subsurface_state.dart';
import 'package:zenith/ui/common/state/surface_state.dart';
import 'package:zenith/ui/common/state/xdg_popup_state.dart';
import 'package:zenith/ui/common/state/xdg_surface_state.dart';
import 'package:zenith/ui/common/state/xdg_toplevel_state.dart';

part '../../../generated/ui/common/state/wayland_state.g.dart';

part '../../../generated/ui/common/state/wayland_state.freezed.dart';

@freezed
class WaylandStates with _$WaylandStates {
  const WaylandStates._();

  const factory WaylandStates({
    required ISet<int> ids,
    required IMap<int, SurfaceState> surfaces,
    required IMap<int, SubsurfaceState> subsurfaces,
    required IMap<int, XdgSurfaceState> xdgSurfaces,
    required IMap<int, XdgToplevelState> xdgToplevels,
    required IMap<int, XdgPopupState> xdgPopups,
  }) = _WaylandStates;

  bool surfaceExists(int id) {
    return ids.contains(id);
  }

  bool isMapped(int id) {
    final surface = surfaces[id]!;

    if (surface.soonDestroyed) return false;

    bool hasTexture = surface.textureId.value != -1;
    if (!hasTexture) return false;

    switch (surface.role) {
      case SurfaceRole.none:
        return false;

      case SurfaceRole.subsurface:
        final parent = subsurfaces[surface.viewId]!.parent;
        return isMapped(parent);

      case SurfaceRole.xdgSurface:
        final role = xdgSurfaces[id]!.role;

        switch (role) {
          case XdgSurfaceRole.none:
            return false;

          case XdgSurfaceRole.toplevel:
          case XdgSurfaceRole.popup:
            return true;
        }
    }
  }

  bool isToplevel(int id) {
    final surface = surfaces[id]!;
    if (surface.role != SurfaceRole.xdgSurface) return false;
    final xdgSurface = xdgSurfaces[id]!;
    return xdgSurface.role == XdgSurfaceRole.toplevel;
  }

  bool isPopup(int id) {
    final surface = surfaces[id]!;
    if (surface.role != SurfaceRole.xdgSurface) return false;
    final xdgSurface = xdgSurfaces[id]!;
    return xdgSurface.role == XdgSurfaceRole.popup;
  }
}

@Riverpod(keepAlive: true)
class WaylandProvider extends _$WaylandProvider {
  @override
  WaylandStates build() {
    return WaylandStates(
      ids: ISet(),
      surfaces: IMap(),
      subsurfaces: IMap(),
      xdgSurfaces: IMap(),
      xdgToplevels: IMap(),
      xdgPopups: IMap(),
    );
  }

  void addSurface(int id) {
    state = state.copyWith(
      ids: state.ids.add(id),
      surfaces: state.surfaces.add(
        id,
        SurfaceState(
          soonDestroyed: false,
          role: SurfaceRole.none,
          viewId: id,
          textureId: TextureId(-1),
          oldTextureId: TextureId(-1),
          surfacePosition: Offset.zero,
          surfaceSize: Size.zero,
          scale: 1,
          textureKey: GlobalKey(),
          subsurfacesBelow: IList(),
          subsurfacesAbove: IList(),
          inputRegion: Rect.zero,
        ),
      ),
    );
  }

  void removeSurface(int id) {
    var state = this.state;

    final surface = state.surfaces[id]!;

    if (surface.role == SurfaceRole.subsurface) {
      state = _removeSubsurface(id, state);
    } else if (surface.role == SurfaceRole.xdgSurface) {
      state = _removeXdgSurface(id, state);
    }

    state = state.copyWith(
      ids: state.ids.remove(id),
      surfaces: state.surfaces.remove(id),
      subsurfaces: state.subsurfaces.remove(id),
      xdgSurfaces: state.xdgSurfaces.remove(id),
      xdgToplevels: state.xdgToplevels.remove(id),
      xdgPopups: state.xdgPopups.remove(id),
    );

    this.state = state;
  }

  void commit(int id, SurfaceCommitData data) {
    _propagateMappedProperty(id, () {
      _commit(id, data);
    });
  }

  void _commit(int id, SurfaceCommitData data) {
    final surfaces = state.surfaces.update(
      id,
      (state) => state.copyWith(
        role: data.role,
        textureId: data.textureId,
        surfacePosition: data.surfacePosition,
        surfaceSize: data.surfaceSize,
        scale: data.scale,
        subsurfacesBelow: data.subsurfacesBelow,
        subsurfacesAbove: data.subsurfacesAbove,
        inputRegion: data.inputRegion,
      ),
    );

    var subsurfaces = state.subsurfaces;

    final subsurfaceCommitData = data.subsurfaceCommitData;
    if (subsurfaceCommitData != null) {
      subsurfaces = subsurfaces.update(
        id,
        (state) => state.copyWith(
          position: subsurfaceCommitData.position,
        ),
      );
    }

    var xdgSurfaces = state.xdgSurfaces;
    var xdgPopups = state.xdgPopups;
    var xdgToplevels = state.xdgToplevels;

    final xdgSurfaceCommitData = data.xdgSurfaceCommitData;
    if (xdgSurfaceCommitData != null) {
      xdgSurfaces = xdgSurfaces.update(
        id,
        (state) => state.copyWith(
          role: xdgSurfaceCommitData.role,
          visibleBounds: xdgSurfaceCommitData.visibleBounds,
        ),
      );

      final xdgToplevelCommitData = xdgSurfaceCommitData.xdgToplevelCommitData;
      if (xdgToplevelCommitData != null) {
        xdgToplevels = xdgToplevels.update(
          id,
          (state) => state.copyWith(
            decoration: xdgToplevelCommitData.decoration,
          ),
        );
      }

      final xdgPopupCommitData = xdgSurfaceCommitData.xdgPopupCommitData;
      if (xdgPopupCommitData != null) {
        xdgPopups = xdgPopups.update(
          id,
          (state) => state.copyWith(
            position: xdgPopupCommitData.position,
          ),
        );
      }
    }

    state = state.copyWith(
      surfaces: surfaces,
      subsurfaces: subsurfaces,
      xdgSurfaces: xdgSurfaces,
      xdgToplevels: xdgToplevels,
      xdgPopups: xdgPopups,
    );
  }

  void _propagateMappedProperty(int id, void Function() f) {
    bool wasMapped = state.isMapped(id);
    bool wasTopLevel = state.isToplevel(id);
    bool wasPopup = state.isPopup(id);

    f();

    bool isMapped = state.isMapped(id);
    bool isTopLevel = state.isToplevel(id);
    bool isPopup = state.isPopup(id);

    if (wasMapped == isMapped) {
      return;
    }

    if (isMapped) {
      if (isTopLevel) {
        windowMappedSink.add(id);
      } else if (isPopup) {
        ref.read(popupStackChildrenProvider.notifier).add(id);
      }
    } else {
      if (wasTopLevel) {
        windowUnmappedSink.add(id);
      } else if (wasPopup) {
        ref.read(popupStackChildrenProvider.notifier).remove(id);
      }
    }
  }

  void unmapSurface(int id) {
    _propagateMappedProperty(id, () {
      state = state.copyWith(
        surfaces: state.surfaces.update(
          id,
          (state) => state.copyWith(
            soonDestroyed: true,
          ),
        ),
      );
    });
  }

  void addSubsurface(int id, int parent) {
    state = state.copyWith(
      subsurfaces: state.subsurfaces.add(
        id,
        SubsurfaceState(
          parent: parent,
          position: Offset.zero,
        ),
      ),
    );
  }

  WaylandStates _removeSubsurface(int id, WaylandStates state) {
    final parent = state.subsurfaces[id]!.parent;

    return state.copyWith(
      subsurfaces: state.subsurfaces.remove(id),
      surfaces: state.surfaces.update(
        parent,
        (state) => state.copyWith(
          subsurfacesBelow: state.subsurfacesBelow.remove(id),
          subsurfacesAbove: state.subsurfacesAbove.remove(id),
        ),
      ),
    );
  }

  void addXdgSurface(int id) {
    state = state.copyWith(
      xdgSurfaces: state.xdgSurfaces.add(
        id,
        XdgSurfaceState(
          role: XdgSurfaceRole.none,
          visibleBounds: Rect.zero,
          popups: IList(),
        ),
      ),
    );
  }

  WaylandStates _removeXdgSurface(int id, WaylandStates state) {
    final xdgSurface = state.xdgSurfaces[id]!;

    if (xdgSurface.role == XdgSurfaceRole.toplevel) {
      state = _removeXdgToplevel(id, state);
    } else if (xdgSurface.role == XdgSurfaceRole.popup) {
      state = _removeXdgPopup(id, state);
    }

    return state.copyWith(
      xdgSurfaces: state.xdgSurfaces.remove(id),
    );
  }

  void addXdgToplevel(int id) {
    final focusNode = FocusNode();

    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.add(
        id,
        XdgToplevelState(
          visible: true,
          virtualKeyboardKey: GlobalKey(),
          focusNode: focusNode,
          interactiveMoveRequested: Object(),
          interactiveResizeRequested: ResizeEdgeObject(ResizeEdge.top),
          decoration: null,
          title: "",
          appId: "",
          tilingRequested: null,
        ),
      ),
    );

    // TODO: Not here.
    focusNode.addListener(() {
      ref
          .read(platformApiProvider.notifier)
          .activateWindow(id, focusNode.hasFocus);
    });
  }

  WaylandStates _removeXdgToplevel(int id, WaylandStates state) {
    return state.copyWith(
      xdgToplevels: state.xdgToplevels.remove(id),
    );
  }

  void addXdgPopup(int id, int parent) {
    state = state.copyWith(
      xdgPopups: state.xdgPopups.add(
        id,
        XdgPopupState(
          parentViewId: parent,
          position: Offset.zero,
          animationsKey: GlobalKey<AnimationsState>(),
          isClosing: false,
        ),
      ),
      xdgSurfaces: state.xdgSurfaces.update(
        parent,
        (state) => state.copyWith(
          popups: state.popups.add(id),
        ),
      ),
    );
  }

  WaylandStates _removeXdgPopup(int id, WaylandStates state) {
    final parent = state.xdgPopups[id]!.parentViewId;

    return state.copyWith(
      xdgPopups: state.xdgPopups.remove(id),
      xdgSurfaces: state.xdgSurfaces.update(
        parent,
        (state) => state.copyWith(
          popups: state.popups.remove(id),
        ),
      ),
    );
  }

  void requestMaximize(int id, bool maximize) {
    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.update(
        id,
        (state) => state.copyWith(
          tilingRequested: maximize ? Tiling.maximized : Tiling.none,
        ),
      ),
    );
  }

  void maximize(int id, bool value) {
    ref.read(platformApiProvider.notifier).maximizeWindow(id, value);
  }

  void resize(int id, int width, int height) {
    ref.read(platformApiProvider.notifier).resizeWindow(id, width, height);
  }

  void requestInteractiveMove(int id) {
    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.update(
        id,
        (state) => state.copyWith(
          interactiveMoveRequested: Object(),
        ),
      ),
    );
  }

  void requestInteractiveResize(int id, ResizeEdge edge) {
    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.update(
        id,
        (state) => state.copyWith(
          interactiveResizeRequested: ResizeEdgeObject(edge),
        ),
      ),
    );
  }

  void setDecoration(int id, ToplevelDecoration decoration) {
    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.update(
        id,
        (state) => state.copyWith(
          decoration: decoration,
        ),
      ),
    );
  }

  void setTitle(int id, String title) {
    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.update(
        id,
        (state) => state.copyWith(
          title: title,
        ),
      ),
    );
  }

  void setAppId(int id, String appId) {
    state = state.copyWith(
      xdgToplevels: state.xdgToplevels.update(
        id,
        (state) => state.copyWith(
          appId: appId,
        ),
      ),
    );
  }
}

@immutable
class SurfaceCommitData {
  final SurfaceRole role;
  final TextureId textureId;
  final Offset surfacePosition;
  final Size surfaceSize;
  final double scale;
  final IList<int> subsurfacesBelow;
  final IList<int> subsurfacesAbove;
  final Rect inputRegion;

  final SubsurfaceCommitData? subsurfaceCommitData;
  final XdgSurfaceCommitData? xdgSurfaceCommitData;

  const SurfaceCommitData({
    required this.role,
    required this.textureId,
    required this.surfacePosition,
    required this.surfaceSize,
    required this.scale,
    required this.subsurfacesBelow,
    required this.subsurfacesAbove,
    required this.inputRegion,
    required this.subsurfaceCommitData,
    required this.xdgSurfaceCommitData,
  });
}

@immutable
class SubsurfaceCommitData {
  final Offset position;

  const SubsurfaceCommitData({
    required this.position,
  });
}

@immutable
class XdgSurfaceCommitData {
  final XdgSurfaceRole role;
  final Rect visibleBounds;

  final XdgPopupCommitData? xdgPopupCommitData;
  final XdgToplevelCommitData? xdgToplevelCommitData;

  const XdgSurfaceCommitData({
    required this.role,
    required this.visibleBounds,
    required this.xdgPopupCommitData,
    required this.xdgToplevelCommitData,
  });
}

@immutable
class XdgToplevelCommitData {
  final ToplevelDecoration? decoration;

  const XdgToplevelCommitData({
    required this.decoration,
  });
}

@immutable
class XdgPopupCommitData {
  final Offset position;

  const XdgPopupCommitData({
    required this.position,
  });
}
