// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of '../../../../ui/common/state/xdg_surface_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$XdgSurfaceState {
  XdgSurfaceRole get role => throw _privateConstructorUsedError;
  Rect get visibleBounds => throw _privateConstructorUsedError;
  IList<int> get popups => throw _privateConstructorUsedError;

  /// Create a copy of XdgSurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $XdgSurfaceStateCopyWith<XdgSurfaceState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $XdgSurfaceStateCopyWith<$Res> {
  factory $XdgSurfaceStateCopyWith(
          XdgSurfaceState value, $Res Function(XdgSurfaceState) then) =
      _$XdgSurfaceStateCopyWithImpl<$Res, XdgSurfaceState>;
  @useResult
  $Res call({XdgSurfaceRole role, Rect visibleBounds, IList<int> popups});
}

/// @nodoc
class _$XdgSurfaceStateCopyWithImpl<$Res, $Val extends XdgSurfaceState>
    implements $XdgSurfaceStateCopyWith<$Res> {
  _$XdgSurfaceStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of XdgSurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? role = null,
    Object? visibleBounds = null,
    Object? popups = null,
  }) {
    return _then(_value.copyWith(
      role: null == role
          ? _value.role
          : role // ignore: cast_nullable_to_non_nullable
              as XdgSurfaceRole,
      visibleBounds: null == visibleBounds
          ? _value.visibleBounds
          : visibleBounds // ignore: cast_nullable_to_non_nullable
              as Rect,
      popups: null == popups
          ? _value.popups
          : popups // ignore: cast_nullable_to_non_nullable
              as IList<int>,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$XdgSurfaceStateImplCopyWith<$Res>
    implements $XdgSurfaceStateCopyWith<$Res> {
  factory _$$XdgSurfaceStateImplCopyWith(_$XdgSurfaceStateImpl value,
          $Res Function(_$XdgSurfaceStateImpl) then) =
      __$$XdgSurfaceStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({XdgSurfaceRole role, Rect visibleBounds, IList<int> popups});
}

/// @nodoc
class __$$XdgSurfaceStateImplCopyWithImpl<$Res>
    extends _$XdgSurfaceStateCopyWithImpl<$Res, _$XdgSurfaceStateImpl>
    implements _$$XdgSurfaceStateImplCopyWith<$Res> {
  __$$XdgSurfaceStateImplCopyWithImpl(
      _$XdgSurfaceStateImpl _value, $Res Function(_$XdgSurfaceStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of XdgSurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? role = null,
    Object? visibleBounds = null,
    Object? popups = null,
  }) {
    return _then(_$XdgSurfaceStateImpl(
      role: null == role
          ? _value.role
          : role // ignore: cast_nullable_to_non_nullable
              as XdgSurfaceRole,
      visibleBounds: null == visibleBounds
          ? _value.visibleBounds
          : visibleBounds // ignore: cast_nullable_to_non_nullable
              as Rect,
      popups: null == popups
          ? _value.popups
          : popups // ignore: cast_nullable_to_non_nullable
              as IList<int>,
    ));
  }
}

/// @nodoc

class _$XdgSurfaceStateImpl
    with DiagnosticableTreeMixin
    implements _XdgSurfaceState {
  const _$XdgSurfaceStateImpl(
      {required this.role, required this.visibleBounds, required this.popups});

  @override
  final XdgSurfaceRole role;
  @override
  final Rect visibleBounds;
  @override
  final IList<int> popups;

  @override
  String toString({DiagnosticLevel minLevel = DiagnosticLevel.info}) {
    return 'XdgSurfaceState(role: $role, visibleBounds: $visibleBounds, popups: $popups)';
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties
      ..add(DiagnosticsProperty('type', 'XdgSurfaceState'))
      ..add(DiagnosticsProperty('role', role))
      ..add(DiagnosticsProperty('visibleBounds', visibleBounds))
      ..add(DiagnosticsProperty('popups', popups));
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$XdgSurfaceStateImpl &&
            (identical(other.role, role) || other.role == role) &&
            (identical(other.visibleBounds, visibleBounds) ||
                other.visibleBounds == visibleBounds) &&
            const DeepCollectionEquality().equals(other.popups, popups));
  }

  @override
  int get hashCode => Object.hash(runtimeType, role, visibleBounds,
      const DeepCollectionEquality().hash(popups));

  /// Create a copy of XdgSurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$XdgSurfaceStateImplCopyWith<_$XdgSurfaceStateImpl> get copyWith =>
      __$$XdgSurfaceStateImplCopyWithImpl<_$XdgSurfaceStateImpl>(
          this, _$identity);
}

abstract class _XdgSurfaceState implements XdgSurfaceState {
  const factory _XdgSurfaceState(
      {required final XdgSurfaceRole role,
      required final Rect visibleBounds,
      required final IList<int> popups}) = _$XdgSurfaceStateImpl;

  @override
  XdgSurfaceRole get role;
  @override
  Rect get visibleBounds;
  @override
  IList<int> get popups;

  /// Create a copy of XdgSurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$XdgSurfaceStateImplCopyWith<_$XdgSurfaceStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
