import 'dart:async';
import 'dart:ffi' show Finalizable;

import 'package:fast_immutable_collections/fast_immutable_collections.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:zenith/ui/common/state/surface_state.dart';
import 'package:zenith/ui/common/state/tasks_provider.dart';
import 'package:zenith/ui/common/state/wayland_state.dart';
import 'package:zenith/ui/common/state/xdg_surface_state.dart';
import 'package:zenith/ui/common/state/xdg_toplevel_state.dart';

part 'generated/platform_api.g.dart';

final _windowMappedController = StreamController<int>.broadcast();
final Stream<int> windowMappedStream = _windowMappedController.stream;
final Sink<int> windowMappedSink = _windowMappedController.sink;

final _windowUnmappedController = StreamController<int>.broadcast();
final Stream<int> windowUnmappedStream = _windowUnmappedController.stream;
final Sink<int> windowUnmappedSink = _windowUnmappedController.sink;

@Riverpod(keepAlive: true)
class WindowMappedStream extends _$WindowMappedStream {
  @override
  Stream<int> build() => windowMappedStream;
}

@Riverpod(keepAlive: true)
class WindowUnmappedStream extends _$WindowUnmappedStream {
  @override
  Stream<int> build() => windowUnmappedStream;
}

@Riverpod(keepAlive: true)
Stream<dynamic> _textInputEventStreamById(Ref ref, int viewId) {
  return ref
      .watch(platformApiProvider)
      .textInputEventsStream
      .where((event) => event["view_id"] == viewId);
}

@Riverpod(keepAlive: true)
Future<TextInputEventType> textInputEventStream(Ref ref, int viewId) async {
  dynamic event =
      await ref.watch(_textInputEventStreamByIdProvider(viewId).future);
  switch (event["type"]) {
    case "enable":
      return TextInputEnable();
    case "disable":
      return TextInputDisable();
    case "commit":
      return TextInputCommit();
    default:
      throw ArgumentError.value(event["type"],
          "Must be 'enable', 'disable', or 'commit'", "event['type']");
  }
}

@Riverpod(keepAlive: true)
class PlatformApi extends _$PlatformApi {
  late final textureFinalizer = Finalizer((int textureId) async {
    // It's possible for a render pass to be late and to use a texture id, even if the object
    // is no longer in memory. Give a generous interval of time for any renders using this texture
    // to finalize.
    await Future.delayed(const Duration(seconds: 1));
    unregisterViewTexture(textureId);
  });

  @override
  PlatformApiState build() {
    return PlatformApiState();
  }

  void init() {
    ref.read(tasksProvider);

    state.platform.setMethodCallHandler((call) async {
      try {
        switch (call.method) {
          case "new_surface":
            _newSurface(call.arguments);
            break;
          case "new_subsurface":
            _newSubsurface(call.arguments);
            break;
          case "new_toplevel":
            _newToplevelSurface(call.arguments);
            break;
          case "new_popup":
            _newPopupSurface(call.arguments);
            break;
          case "commit_surface":
            _commitSurface(call.arguments);
            break;
          case "send_text_input_event":
            _sendTextInputEvent(call.arguments);
            break;
          case "interactive_move":
            _interactiveMove(call.arguments);
            break;
          case "interactive_resize":
            _interactiveResize(call.arguments);
            break;
          case "set_title":
            _setTitle(call.arguments);
            break;
          case "set_app_id":
            _setAppId(call.arguments);
            break;
          case "request_maximize":
            _requestMaximize(call.arguments);
            break;
          case "destroy_surface":
            _destroySurface(call.arguments);
            break;
          default:
            throw PlatformException(
              code: "unknown_method",
              message: "Unknown method ${call.method}",
            );
        }
      } catch (e, s) {
        FlutterError.reportError(FlutterErrorDetails(exception: e, stack: s));
        rethrow;
      }
    });
  }

  Future<void> startupComplete() {
    return state.platform.invokeMethod("startup_complete");
  }

  Future<void> pointerHoversView(int viewId, Offset position) {
    return state.platform.invokeMethod("pointer_hover", {
      "view_id": viewId,
      "x": position.dx,
      "y": position.dy,
    });
  }

  Future<void> sendMouseButtonsEventToView(int buttons, bool isPressed) {
    // One might find surprising that the view id is not sent to the platform. This is because the view id is only sent
    // when the pointer moves, and when a button event happens, the platform already knows which view it hovers.
    return state.platform.invokeMethod("mouse_button_event", {
      "buttons": buttons,
      "is_pressed": isPressed,
    });
  }

  Future<void> pointerExitsView() {
    return state.platform.invokeMethod("pointer_exit");
  }

  Future<void> activateWindow(int viewId, bool activate) {
    return state.platform.invokeMethod('activate_window', [viewId, activate]);
  }

  Future<void> changeWindowVisibility(int viewId, bool visible) {
    return state.platform.invokeMethod('change_window_visibility', {
      "view_id": viewId,
      "visible": visible,
    });
  }

  Future<void> unregisterViewTexture(int textureId) {
    return state.platform.invokeMethod('unregister_view_texture', {
      "texture_id": textureId,
    });
  }

  Future<void> touchDown(int viewId, int touchId, Offset position) {
    return state.platform.invokeMethod('touch_down', {
      "view_id": viewId,
      "touch_id": touchId,
      "x": position.dx,
      "y": position.dy,
    });
  }

  Future<void> touchMotion(int touchId, Offset position) {
    return state.platform.invokeMethod('touch_motion', {
      "touch_id": touchId,
      "x": position.dx,
      "y": position.dy,
    });
  }

  Future<void> touchUp(int touchId) {
    return state.platform.invokeMethod('touch_up', {
      "touch_id": touchId,
    });
  }

  Future<void> touchCancel(int touchId) {
    return state.platform.invokeMethod('touch_cancel', {
      "touch_id": touchId,
    });
  }

  Future<void> insertText(int viewId, String text) {
    return state.platform.invokeMethod('insert_text', {
      "view_id": viewId,
      "text": text,
    });
  }

  Future<void> emulateKeyCode(int viewId, int keyCode) {
    return state.platform.invokeMethod('emulate_keycode', {
      "view_id": viewId,
      "keycode": keyCode,
    });
  }

  Future<void> openWindowsMaximized(bool value) {
    return state.platform.invokeMethod("open_windows_maximized", value);
  }

  Future<void> maximizedWindowSize(int width, int height) {
    return state.platform.invokeMethod("maximized_window_size", {
      "width": width,
      "height": height,
    });
  }

  Future<void> maximizeWindow(int viewId, bool value) {
    return state.platform.invokeMethod("maximize_window", {
      "view_id": viewId,
      "value": value,
    });
  }

  Future<void> resizeWindow(int viewId, int width, int height) {
    return state.platform.invokeMethod("resize_window", {
      "view_id": viewId,
      "width": width,
      "height": height,
    });
  }

  Stream<TextInputEventType> getTextInputEventsForViewId(int viewId) {
    return state.textInputEventsStream
        .where((event) => event["view_id"] == viewId)
        .map((event) {
      switch (event["type"]) {
        case "enable":
          return TextInputEnable();
        case "disable":
          return TextInputDisable();
        case "commit":
          return TextInputCommit();
        default:
          throw ArgumentError.value(event["type"],
              "Must be 'enable', 'disable', or 'commit'", "event['type']");
      }
    });
  }

  Future<void> closeView(int viewId) {
    return state.platform.invokeMethod("close_window", {
      "view_id": viewId,
    });
  }

  Future<AuthenticationResponse> unlockSession(String password) async {
    Map<String, dynamic>? response =
        await state.platform.invokeMapMethod("unlock_session", {
      "password": password,
    });
    if (response == null) {
      return AuthenticationResponse(false, "");
    }
    return AuthenticationResponse(
        response["success"] as bool, response["message"] as String);
  }

  /// The display will not generate frame events anymore if it's disabled, meaning that rendering is stopped.
  Future<void> enableDisplay(bool enable) async {
    return state.platform.invokeMethod("enable_display", {
      "enable": enable,
    });
  }

  void _newSurface(dynamic event) {
    int viewId = event["view_id"];
    ref.read(waylandProviderProvider.notifier).addSurface(viewId);
  }

  void _newSubsurface(dynamic event) {
    int viewId = event["view_id"];
    int parent = event["parent"];
    ref.read(waylandProviderProvider.notifier).addSubsurface(viewId, parent);
  }

  void _newToplevelSurface(dynamic event) {
    int viewId = event["view_id"];
    ref.read(waylandProviderProvider.notifier).addXdgSurface(viewId);
    ref.read(waylandProviderProvider.notifier).addXdgToplevel(viewId);
  }

  void _newPopupSurface(dynamic event) {
    int viewId = event["view_id"];
    int parent = event["parent"];
    ref.read(waylandProviderProvider.notifier).addXdgSurface(viewId);
    ref.read(waylandProviderProvider.notifier).addXdgPopup(viewId, parent);
  }

  void _commitSurface(dynamic event) {
    int viewId = event["view_id"];

    // for (int id in subsurfaceIdsBelow) {
    //   ref.read(subsurfaceStatesProvider(id).notifier).set_parent(viewId);
    // }
    //
    // for (int id in subsurfaceIdsAbove) {
    //   ref.read(subsurfaceStatesProvider(id).notifier).set_parent(viewId);
    // }

    SurfaceCommitData commitData = _parseSurfaceCommitData(event);
    ref.read(waylandProviderProvider.notifier).commit(viewId, commitData);

    // bool hasToplevelDecoration = event["has_toplevel_decoration"];
    // if (hasToplevelDecoration) {
    //   int toplevelDecorationInt = event["toplevel_decoration"];
    //   var decoration = ToplevelDecoration.fromInt(toplevelDecorationInt);
    //   ref
    //       .read(xdgToplevelStatesProvider(viewId).notifier)
    //       .setDecoration(decoration);
    // }
    //
    // bool hasToplevelTitle = event["has_toplevel_title"];
    // if (hasToplevelTitle) {
    //   String title = event["toplevel_title"];
    //   ref.read(xdgToplevelStatesProvider(viewId).notifier).setTitle(title);
    // }
    //
    // bool hasToplevelAppId = event["has_toplevel_app_id"];
    // if (hasToplevelAppId) {
    //   String appId = event["toplevel_app_id"];
    //   ref.read(xdgToplevelStatesProvider(viewId).notifier).setAppId(appId);
    // }
  }

  SurfaceCommitData _parseSurfaceCommitData(dynamic data) {
    int viewId = data["view_id"];
    dynamic surface = data["surface"];
    final role = SurfaceRole.values[surface["role"]];
    dynamic roleState = surface["role_state"];

    int textureIdInt = surface["textureId"];
    // TODO: Don't remove the late keyword even if it still compiles !
    // If you remove it, it will run correctly in debug mode but not in release mode.
    // I should make a minimum reproducible example and file a bug.
    late TextureId textureId;

    TextureId currentTextureId =
        ref.read(waylandProviderProvider).surfaces[viewId]!.textureId;
    if (textureIdInt == currentTextureId.value) {
      textureId = currentTextureId;
    } else {
      textureId = TextureId(textureIdInt);
      textureFinalizer.attach(textureId, textureId.value, detach: textureId);
    }

    int x = surface["x"];
    int y = surface["y"];
    int width = surface["width"];
    int height = surface["height"];
    int scale = surface["scale"];

    dynamic inputRegion = surface["input_region"];
    int left = inputRegion["x1"];
    int top = inputRegion["y1"];
    int right = inputRegion["x2"];
    int bottom = inputRegion["y2"];
    var inputRegionRect = Rect.fromLTRB(
      left.toDouble(),
      top.toDouble(),
      right.toDouble(),
      bottom.toDouble(),
    );

    List<dynamic> subsurfaceBelow = surface["subsurfaces_below"];
    List<dynamic> subsurfaceAbove = surface["subsurfaces_above"];

    final subsurfaceIdsBelow = subsurfaceBelow.cast<int>().lockUnsafe;
    final subsurfaceIdsAbove = subsurfaceAbove.cast<int>().lockUnsafe;

    return SurfaceCommitData(
      role: role,
      textureId: textureId,
      surfacePosition: Offset(x.toDouble(), y.toDouble()),
      surfaceSize: Size(width.toDouble(), height.toDouble()),
      scale: scale.toDouble(),
      subsurfacesBelow: subsurfaceIdsBelow,
      subsurfacesAbove: subsurfaceIdsAbove,
      inputRegion: inputRegionRect,
      subsurfaceCommitData: role == SurfaceRole.subsurface
          ? _parseSubsurfaceCommitData(roleState)
          : null,
      xdgSurfaceCommitData: role == SurfaceRole.xdgSurface
          ? _parseXdgSurfaceCommitData(roleState)
          : null,
    );
  }

  SubsurfaceCommitData? _parseSubsurfaceCommitData(dynamic subsurface) {
    int x = subsurface["x"];
    int y = subsurface["y"];
    var position = Offset(x.toDouble(), y.toDouble());
    return SubsurfaceCommitData(
      position: position,
    );
  }

  XdgSurfaceCommitData? _parseXdgSurfaceCommitData(dynamic xdgSurface) {
    final role = XdgSurfaceRole.values[xdgSurface["role"]];
    dynamic roleState = xdgSurface["role_state"];
    int x = xdgSurface["x"];
    int y = xdgSurface["y"];
    int width = xdgSurface["width"];
    int height = xdgSurface["height"];
    var visibleBounds = Rect.fromLTWH(
      x.toDouble(),
      y.toDouble(),
      width.toDouble(),
      height.toDouble(),
    );
    return XdgSurfaceCommitData(
      role: role,
      visibleBounds: visibleBounds,
      xdgToplevelCommitData: role == XdgSurfaceRole.toplevel
          ? _parseXdgToplevelCommitData(roleState)
          : null,
      xdgPopupCommitData: role == XdgSurfaceRole.popup
          ? _parseXdgPopupCommitData(roleState)
          : null,
    );
  }

  XdgToplevelCommitData? _parseXdgToplevelCommitData(dynamic xdgToplevel) {
    int? decorationInt = xdgToplevel["decoration"];
    final decoration = decorationInt == null
        ? null
        : ToplevelDecoration.fromInt(decorationInt);

    return XdgToplevelCommitData(
      decoration: decoration,
    );
  }

  XdgPopupCommitData? _parseXdgPopupCommitData(dynamic xdgPopup) {
    int x = xdgPopup["x"];
    int y = xdgPopup["y"];
    var position = Offset(x.toDouble(), y.toDouble());

    return XdgPopupCommitData(
      position: position,
    );
  }

  void _sendTextInputEvent(dynamic event) {
    state.textInputEventsSink.add(event);
  }

  void _interactiveMove(dynamic event) {
    int viewId = event["view_id"];
    ref.read(waylandProviderProvider.notifier).requestInteractiveMove(viewId);
  }

  void _interactiveResize(dynamic event) {
    int viewId = event["view_id"];
    int edge = event["edge"];

    ResizeEdge resizeEdge = ResizeEdge.fromInt(edge);

    ref
        .read(waylandProviderProvider.notifier)
        .requestInteractiveResize(viewId, resizeEdge);
  }

  void _setTitle(dynamic event) {
    int viewId = event["view_id"];
    String title = event["title"];
    ref.read(waylandProviderProvider.notifier).setTitle(viewId, title);
  }

  void _setAppId(dynamic event) {
    int viewId = event["view_id"];
    String appId = event["app_id"];
    ref.read(waylandProviderProvider.notifier).setAppId(viewId, appId);
  }

  void _requestMaximize(dynamic event) {
    int viewId = event["view_id"];
    bool maximize = event["maximize"];
    ref
        .read(waylandProviderProvider.notifier)
        .requestMaximize(viewId, maximize);
  }

  void _destroySurface(dynamic event) async {
    int viewId = event["view_id"];

    ref.read(waylandProviderProvider.notifier).unmapSurface(viewId);

    // TODO: Find a better way. Maybe store subscriptions in a list.
    // 3 sec is more than enough for any close animations.
    await Future.delayed(const Duration(seconds: 3));

    ref.read(waylandProviderProvider.notifier).removeSurface(viewId);
  }

  Future<void> hideKeyboard(int viewId) {
    return state.platform.invokeMethod('hide_keyboard', {
      "view_id": viewId,
    });
  }
}

class PlatformApiState {
  final _textInputEventsStreamController =
      StreamController<dynamic>.broadcast();
  late final Stream<dynamic> textInputEventsStream;
  late final Sink<dynamic> textInputEventsSink;

  MethodChannel platform = const MethodChannel('platform');

  PlatformApiState() {
    textInputEventsStream = _textInputEventsStreamController.stream;
    textInputEventsSink = _textInputEventsStreamController.sink;
  }
}

abstract class TextInputEventType {}

class TextInputEnable extends TextInputEventType {}

class TextInputDisable extends TextInputEventType {}

class TextInputCommit extends TextInputEventType {}

class AuthenticationResponse {
  AuthenticationResponse(this.success, this.message);

  bool success;
  String message;
}

class TextureId implements Finalizable {
  final int value;

  TextureId(this.value);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is TextureId &&
          runtimeType == other.runtimeType &&
          value == other.value);

  @override
  int get hashCode => value.hashCode;
}
