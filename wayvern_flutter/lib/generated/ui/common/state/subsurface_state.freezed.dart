// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of '../../../../ui/common/state/subsurface_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$SubsurfaceState {
  int get parent => throw _privateConstructorUsedError;
  Offset get position => throw _privateConstructorUsedError;

  /// Create a copy of SubsurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SubsurfaceStateCopyWith<SubsurfaceState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SubsurfaceStateCopyWith<$Res> {
  factory $SubsurfaceStateCopyWith(
          SubsurfaceState value, $Res Function(SubsurfaceState) then) =
      _$SubsurfaceStateCopyWithImpl<$Res, SubsurfaceState>;
  @useResult
  $Res call({int parent, Offset position});
}

/// @nodoc
class _$SubsurfaceStateCopyWithImpl<$Res, $Val extends SubsurfaceState>
    implements $SubsurfaceStateCopyWith<$Res> {
  _$SubsurfaceStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SubsurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? parent = null,
    Object? position = null,
  }) {
    return _then(_value.copyWith(
      parent: null == parent
          ? _value.parent
          : parent // ignore: cast_nullable_to_non_nullable
              as int,
      position: null == position
          ? _value.position
          : position // ignore: cast_nullable_to_non_nullable
              as Offset,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SubsurfaceStateImplCopyWith<$Res>
    implements $SubsurfaceStateCopyWith<$Res> {
  factory _$$SubsurfaceStateImplCopyWith(_$SubsurfaceStateImpl value,
          $Res Function(_$SubsurfaceStateImpl) then) =
      __$$SubsurfaceStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({int parent, Offset position});
}

/// @nodoc
class __$$SubsurfaceStateImplCopyWithImpl<$Res>
    extends _$SubsurfaceStateCopyWithImpl<$Res, _$SubsurfaceStateImpl>
    implements _$$SubsurfaceStateImplCopyWith<$Res> {
  __$$SubsurfaceStateImplCopyWithImpl(
      _$SubsurfaceStateImpl _value, $Res Function(_$SubsurfaceStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of SubsurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? parent = null,
    Object? position = null,
  }) {
    return _then(_$SubsurfaceStateImpl(
      parent: null == parent
          ? _value.parent
          : parent // ignore: cast_nullable_to_non_nullable
              as int,
      position: null == position
          ? _value.position
          : position // ignore: cast_nullable_to_non_nullable
              as Offset,
    ));
  }
}

/// @nodoc

class _$SubsurfaceStateImpl implements _SubsurfaceState {
  const _$SubsurfaceStateImpl({required this.parent, required this.position});

  @override
  final int parent;
  @override
  final Offset position;

  @override
  String toString() {
    return 'SubsurfaceState(parent: $parent, position: $position)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SubsurfaceStateImpl &&
            (identical(other.parent, parent) || other.parent == parent) &&
            (identical(other.position, position) ||
                other.position == position));
  }

  @override
  int get hashCode => Object.hash(runtimeType, parent, position);

  /// Create a copy of SubsurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SubsurfaceStateImplCopyWith<_$SubsurfaceStateImpl> get copyWith =>
      __$$SubsurfaceStateImplCopyWithImpl<_$SubsurfaceStateImpl>(
          this, _$identity);
}

abstract class _SubsurfaceState implements SubsurfaceState {
  const factory _SubsurfaceState(
      {required final int parent,
      required final Offset position}) = _$SubsurfaceStateImpl;

  @override
  int get parent;
  @override
  Offset get position;

  /// Create a copy of SubsurfaceState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SubsurfaceStateImplCopyWith<_$SubsurfaceStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
