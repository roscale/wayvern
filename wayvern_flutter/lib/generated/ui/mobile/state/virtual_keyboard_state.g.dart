// GENERATED CODE - DO NOT MODIFY BY HAND

part of '../../../../ui/mobile/state/virtual_keyboard_state.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$virtualKeyboardStateNotifierHash() =>
    r'92da59f7e63883ea311ccb112e62bb58920c14fd';

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

abstract class _$VirtualKeyboardStateNotifier
    extends BuildlessAutoDisposeNotifier<VirtualKeyboardState> {
  late final int viewId;

  VirtualKeyboardState build(
    int viewId,
  );
}

/// See also [VirtualKeyboardStateNotifier].
@ProviderFor(VirtualKeyboardStateNotifier)
const virtualKeyboardStateNotifierProvider =
    VirtualKeyboardStateNotifierFamily();

/// See also [VirtualKeyboardStateNotifier].
class VirtualKeyboardStateNotifierFamily extends Family<VirtualKeyboardState> {
  /// See also [VirtualKeyboardStateNotifier].
  const VirtualKeyboardStateNotifierFamily();

  /// See also [VirtualKeyboardStateNotifier].
  VirtualKeyboardStateNotifierProvider call(
    int viewId,
  ) {
    return VirtualKeyboardStateNotifierProvider(
      viewId,
    );
  }

  @override
  VirtualKeyboardStateNotifierProvider getProviderOverride(
    covariant VirtualKeyboardStateNotifierProvider provider,
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
  String? get name => r'virtualKeyboardStateNotifierProvider';
}

/// See also [VirtualKeyboardStateNotifier].
class VirtualKeyboardStateNotifierProvider
    extends AutoDisposeNotifierProviderImpl<VirtualKeyboardStateNotifier,
        VirtualKeyboardState> {
  /// See also [VirtualKeyboardStateNotifier].
  VirtualKeyboardStateNotifierProvider(
    int viewId,
  ) : this._internal(
          () => VirtualKeyboardStateNotifier()..viewId = viewId,
          from: virtualKeyboardStateNotifierProvider,
          name: r'virtualKeyboardStateNotifierProvider',
          debugGetCreateSourceHash:
              const bool.fromEnvironment('dart.vm.product')
                  ? null
                  : _$virtualKeyboardStateNotifierHash,
          dependencies: VirtualKeyboardStateNotifierFamily._dependencies,
          allTransitiveDependencies:
              VirtualKeyboardStateNotifierFamily._allTransitiveDependencies,
          viewId: viewId,
        );

  VirtualKeyboardStateNotifierProvider._internal(
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
  VirtualKeyboardState runNotifierBuild(
    covariant VirtualKeyboardStateNotifier notifier,
  ) {
    return notifier.build(
      viewId,
    );
  }

  @override
  Override overrideWith(VirtualKeyboardStateNotifier Function() create) {
    return ProviderOverride(
      origin: this,
      override: VirtualKeyboardStateNotifierProvider._internal(
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
  AutoDisposeNotifierProviderElement<VirtualKeyboardStateNotifier,
      VirtualKeyboardState> createElement() {
    return _VirtualKeyboardStateNotifierProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is VirtualKeyboardStateNotifierProvider &&
        other.viewId == viewId;
  }

  @override
  int get hashCode {
    var hash = _SystemHash.combine(0, runtimeType.hashCode);
    hash = _SystemHash.combine(hash, viewId.hashCode);

    return _SystemHash.finish(hash);
  }
}

mixin VirtualKeyboardStateNotifierRef
    on AutoDisposeNotifierProviderRef<VirtualKeyboardState> {
  /// The parameter `viewId` of this provider.
  int get viewId;
}

class _VirtualKeyboardStateNotifierProviderElement
    extends AutoDisposeNotifierProviderElement<VirtualKeyboardStateNotifier,
        VirtualKeyboardState> with VirtualKeyboardStateNotifierRef {
  _VirtualKeyboardStateNotifierProviderElement(super.provider);

  @override
  int get viewId => (origin as VirtualKeyboardStateNotifierProvider).viewId;
}
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member
