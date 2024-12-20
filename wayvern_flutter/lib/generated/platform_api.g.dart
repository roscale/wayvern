// GENERATED CODE - DO NOT MODIFY BY HAND

part of '../platform_api.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

String _$textInputEventStreamByIdHash() =>
    r'a73a40883174b54068c1f2c750106f78ce98e647';

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

/// See also [_textInputEventStreamById].
@ProviderFor(_textInputEventStreamById)
const _textInputEventStreamByIdProvider = _TextInputEventStreamByIdFamily();

/// See also [_textInputEventStreamById].
class _TextInputEventStreamByIdFamily extends Family<AsyncValue<dynamic>> {
  /// See also [_textInputEventStreamById].
  const _TextInputEventStreamByIdFamily();

  /// See also [_textInputEventStreamById].
  _TextInputEventStreamByIdProvider call(
    int viewId,
  ) {
    return _TextInputEventStreamByIdProvider(
      viewId,
    );
  }

  @override
  _TextInputEventStreamByIdProvider getProviderOverride(
    covariant _TextInputEventStreamByIdProvider provider,
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
  String? get name => r'_textInputEventStreamByIdProvider';
}

/// See also [_textInputEventStreamById].
class _TextInputEventStreamByIdProvider extends StreamProvider<dynamic> {
  /// See also [_textInputEventStreamById].
  _TextInputEventStreamByIdProvider(
    int viewId,
  ) : this._internal(
          (ref) => _textInputEventStreamById(
            ref as _TextInputEventStreamByIdRef,
            viewId,
          ),
          from: _textInputEventStreamByIdProvider,
          name: r'_textInputEventStreamByIdProvider',
          debugGetCreateSourceHash:
              const bool.fromEnvironment('dart.vm.product')
                  ? null
                  : _$textInputEventStreamByIdHash,
          dependencies: _TextInputEventStreamByIdFamily._dependencies,
          allTransitiveDependencies:
              _TextInputEventStreamByIdFamily._allTransitiveDependencies,
          viewId: viewId,
        );

  _TextInputEventStreamByIdProvider._internal(
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
  Override overrideWith(
    Stream<dynamic> Function(_TextInputEventStreamByIdRef provider) create,
  ) {
    return ProviderOverride(
      origin: this,
      override: _TextInputEventStreamByIdProvider._internal(
        (ref) => create(ref as _TextInputEventStreamByIdRef),
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
  StreamProviderElement<dynamic> createElement() {
    return _TextInputEventStreamByIdProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is _TextInputEventStreamByIdProvider && other.viewId == viewId;
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
mixin _TextInputEventStreamByIdRef on StreamProviderRef<dynamic> {
  /// The parameter `viewId` of this provider.
  int get viewId;
}

class _TextInputEventStreamByIdProviderElement
    extends StreamProviderElement<dynamic> with _TextInputEventStreamByIdRef {
  _TextInputEventStreamByIdProviderElement(super.provider);

  @override
  int get viewId => (origin as _TextInputEventStreamByIdProvider).viewId;
}

String _$textInputEventStreamHash() =>
    r'70dfe98a2dd76ba7c8754724471c049201bc60d9';

/// See also [textInputEventStream].
@ProviderFor(textInputEventStream)
const textInputEventStreamProvider = TextInputEventStreamFamily();

/// See also [textInputEventStream].
class TextInputEventStreamFamily
    extends Family<AsyncValue<TextInputEventType>> {
  /// See also [textInputEventStream].
  const TextInputEventStreamFamily();

  /// See also [textInputEventStream].
  TextInputEventStreamProvider call(
    int viewId,
  ) {
    return TextInputEventStreamProvider(
      viewId,
    );
  }

  @override
  TextInputEventStreamProvider getProviderOverride(
    covariant TextInputEventStreamProvider provider,
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
  String? get name => r'textInputEventStreamProvider';
}

/// See also [textInputEventStream].
class TextInputEventStreamProvider extends FutureProvider<TextInputEventType> {
  /// See also [textInputEventStream].
  TextInputEventStreamProvider(
    int viewId,
  ) : this._internal(
          (ref) => textInputEventStream(
            ref as TextInputEventStreamRef,
            viewId,
          ),
          from: textInputEventStreamProvider,
          name: r'textInputEventStreamProvider',
          debugGetCreateSourceHash:
              const bool.fromEnvironment('dart.vm.product')
                  ? null
                  : _$textInputEventStreamHash,
          dependencies: TextInputEventStreamFamily._dependencies,
          allTransitiveDependencies:
              TextInputEventStreamFamily._allTransitiveDependencies,
          viewId: viewId,
        );

  TextInputEventStreamProvider._internal(
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
  Override overrideWith(
    FutureOr<TextInputEventType> Function(TextInputEventStreamRef provider)
        create,
  ) {
    return ProviderOverride(
      origin: this,
      override: TextInputEventStreamProvider._internal(
        (ref) => create(ref as TextInputEventStreamRef),
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
  FutureProviderElement<TextInputEventType> createElement() {
    return _TextInputEventStreamProviderElement(this);
  }

  @override
  bool operator ==(Object other) {
    return other is TextInputEventStreamProvider && other.viewId == viewId;
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
mixin TextInputEventStreamRef on FutureProviderRef<TextInputEventType> {
  /// The parameter `viewId` of this provider.
  int get viewId;
}

class _TextInputEventStreamProviderElement
    extends FutureProviderElement<TextInputEventType>
    with TextInputEventStreamRef {
  _TextInputEventStreamProviderElement(super.provider);

  @override
  int get viewId => (origin as TextInputEventStreamProvider).viewId;
}

String _$windowMappedStreamHash() =>
    r'02cb2bcc04157b37d4deac3bb7b131de8915221e';

/// See also [WindowMappedStream].
@ProviderFor(WindowMappedStream)
final windowMappedStreamProvider =
    StreamNotifierProvider<WindowMappedStream, int>.internal(
  WindowMappedStream.new,
  name: r'windowMappedStreamProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$windowMappedStreamHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$WindowMappedStream = StreamNotifier<int>;
String _$windowUnmappedStreamHash() =>
    r'd1daaf5169d1c843996c6fa25f4d44b11be80814';

/// See also [WindowUnmappedStream].
@ProviderFor(WindowUnmappedStream)
final windowUnmappedStreamProvider =
    StreamNotifierProvider<WindowUnmappedStream, int>.internal(
  WindowUnmappedStream.new,
  name: r'windowUnmappedStreamProvider',
  debugGetCreateSourceHash: const bool.fromEnvironment('dart.vm.product')
      ? null
      : _$windowUnmappedStreamHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$WindowUnmappedStream = StreamNotifier<int>;
String _$platformApiHash() => r'df6f4ca40e03ab91490730e7f43cf66afe2a50f4';

/// See also [PlatformApi].
@ProviderFor(PlatformApi)
final platformApiProvider =
    NotifierProvider<PlatformApi, PlatformApiState>.internal(
  PlatformApi.new,
  name: r'platformApiProvider',
  debugGetCreateSourceHash:
      const bool.fromEnvironment('dart.vm.product') ? null : _$platformApiHash,
  dependencies: null,
  allTransitiveDependencies: null,
);

typedef _$PlatformApi = Notifier<PlatformApiState>;
// ignore_for_file: type=lint
// ignore_for_file: subtype_of_sealed_class, invalid_use_of_internal_member, invalid_use_of_visible_for_testing_member, deprecated_member_use_from_same_package
