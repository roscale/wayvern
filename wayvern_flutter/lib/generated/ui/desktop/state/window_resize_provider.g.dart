// GENERATED CODE - DO NOT MODIFY BY HAND

part of '../../../../ui/desktop/state/window_resize_provider.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$windowResizeHash() => r'8470126cda7504cd9e09da52240ef174176a72d6';

/// Copied from Dart SDK
class _SystemHash {
  _SystemHash._();

  static int combine(int hash, int value) {
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + value);
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + ((0x0007ffff & hash) << 10));
    return hash ^ (hash >> 6);
  }

  static int finish(int hash) {
    // ignore: parameter_assignments
    hash = 0x1fffffff & (hash + ((0x03ffffff & hash) << 3));
    // ignore: parameter_assignments
    hash = hash ^ (hash >> 11);
    return 0x1fffffff & (hash + ((0x00003fff & hash) << 15));
  }
}

abstract class _$WindowResize extends BuildlessNotifier<ResizerState> {
  late final int viewId;

  ResizerState build(
    int viewId,
  );
}

/// See also [WindowResize].
@ProviderFor(WindowResize)
const windowResizeProvider = WindowResizeFamily();

/// See also [WindowResize].
class WindowResizeFamily extends Family<ResizerState> {
  /// See also [WindowResize].
  const WindowResizeFamily();

  /// See also [WindowResize].
  WindowResizeProvider call(
    int viewId,
  ) {
    return WindowResizeProvider(
      viewId,
    );
  }

  @override
  WindowResizeProvider getProviderOverride(
    covariant WindowResizeProvider provider,
  ) {
    return call(
      provider.viewId,
    );
  }

  static const Iterable<ProviderOrFamily>? _dependencies = null;

  @override
  Iterable<ProviderOrFamily>? get dependencies => _dependencies;

  static const Iterable<ProviderOrFamily>? _allTransitiveDependencies = null;

  @override
  Iterable<ProviderOrFamily>? get allTransitiveDependencies =>
      _allTransitiveDependencies;

  @override
  String? get name => r'windowResizeProvider';
}

/// See also [WindowResize].
class WindowResizeProvider
    extends NotifierProviderImpl<WindowResize, ResizerState> {
  /// See also [WindowResize].
  WindowResizeProvider(
    int viewId,
  ) : this._internal(
          () => WindowResize()..viewId = viewId,
          from: windowResizeProvider,
          name: r'windowResizeProvider',
          debugGetCreateSourceHash:
              const bool.fromEnvironment('dart.vm.product')
                  ? null
                  : _$windowResizeHash,
          dependencies: WindowResizeFamily._dependencies,
          allTransitiveDependencies:
              WindowResizeFamily._allTransitiveDependencies,
          viewId: viewId,
        );

  WindowResizeProvider._internal(
    super._createNotifier, {
    required super.name,
    required super.dependencies,
    required super.allTransitiveDependencies,
    required super.debugGetCreateSourceHash,
    required super.from,
    required this.viewId,
  }) : super.internal();

  final int viewId;

  @override
  ResizerState runNotifierBuild(
    covariant WindowResize notifier,
  ) {
    return notifier.build(
      viewId,
    );
  }

  @override
  Override overrideWith(WindowResize Function() create) {
    return ProviderOverride(
      origin: this,
      override: WindowResizeProvider._internal(
        () => create()..viewId = viewId,
        from: from,
        name: null,
        dependencies: null,
        allTransitiveDependencies: null,
        debugGetCreateSourceHash: null,
        viewId: viewId,
      ),
    );
  }

  @override
  NotifierProviderElement<WindowResize, ResizerState> createElement() {
    return _WindowResizeProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is WindowResizeProvider && other.viewId == viewId;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, viewId.hashCode);

    return _SystemHash.finish(hash);
  }
}

@Deprecated('Will be removed in 3.0. Use Ref instead')
// ignore: unused_element
mixin WindowResizeRef on NotifierProviderRef<ResizerState> {
  /// The parameter `viewId` of this provider.
  int get viewId;
}

class _WindowResizeProviderElement
    extends NotifierProviderElement<WindowResize, ResizerState>
    with WindowResizeRef {
  _WindowResizeProviderElement(super.provider);

  @override
  int get viewId => (origin as WindowResizeProvider).viewId;
}
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
