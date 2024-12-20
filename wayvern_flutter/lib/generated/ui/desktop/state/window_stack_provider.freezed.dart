// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of '../../../../ui/desktop/state/window_stack_provider.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$WindowStackState {
  IList<int> get stack => throw _privateConstructorUsedError;
  ISet<int> get animateClosing => throw _privateConstructorUsedError;
  Size get desktopSize => throw _privateConstructorUsedError;

  /// Create a copy of WindowStackState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $WindowStackStateCopyWith<WindowStackState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $WindowStackStateCopyWith<$Res> {
  factory $WindowStackStateCopyWith(
          WindowStackState value, $Res Function(WindowStackState) then) =
      _$WindowStackStateCopyWithImpl<$Res, WindowStackState>;
  @useResult
  $Res call({IList<int> stack, ISet<int> animateClosing, Size desktopSize});
}

/// @nodoc
class _$WindowStackStateCopyWithImpl<$Res, $Val extends WindowStackState>
    implements $WindowStackStateCopyWith<$Res> {
  _$WindowStackStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of WindowStackState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? stack = null,
    Object? animateClosing = null,
    Object? desktopSize = null,
  }) {
    return _then(_value.copyWith(
      stack: null == stack
          ? _value.stack
          : stack // ignore: cast_nullable_to_non_nullable
              as IList<int>,
      animateClosing: null == animateClosing
          ? _value.animateClosing
          : animateClosing // ignore: cast_nullable_to_non_nullable
              as ISet<int>,
      desktopSize: null == desktopSize
          ? _value.desktopSize
          : desktopSize // ignore: cast_nullable_to_non_nullable
              as Size,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$WindowStackStateImplCopyWith<$Res>
    implements $WindowStackStateCopyWith<$Res> {
  factory _$$WindowStackStateImplCopyWith(_$WindowStackStateImpl value,
          $Res Function(_$WindowStackStateImpl) then) =
      __$$WindowStackStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({IList<int> stack, ISet<int> animateClosing, Size desktopSize});
}

/// @nodoc
class __$$WindowStackStateImplCopyWithImpl<$Res>
    extends _$WindowStackStateCopyWithImpl<$Res, _$WindowStackStateImpl>
    implements _$$WindowStackStateImplCopyWith<$Res> {
  __$$WindowStackStateImplCopyWithImpl(_$WindowStackStateImpl _value,
      $Res Function(_$WindowStackStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of WindowStackState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? stack = null,
    Object? animateClosing = null,
    Object? desktopSize = null,
  }) {
    return _then(_$WindowStackStateImpl(
      stack: null == stack
          ? _value.stack
          : stack // ignore: cast_nullable_to_non_nullable
              as IList<int>,
      animateClosing: null == animateClosing
          ? _value.animateClosing
          : animateClosing // ignore: cast_nullable_to_non_nullable
              as ISet<int>,
      desktopSize: null == desktopSize
          ? _value.desktopSize
          : desktopSize // ignore: cast_nullable_to_non_nullable
              as Size,
    ));
  }
}

/// @nodoc

class _$WindowStackStateImpl extends _WindowStackState {
  const _$WindowStackStateImpl(
      {required this.stack,
      required this.animateClosing,
      required this.desktopSize})
      : super._();

  @override
  final IList<int> stack;
  @override
  final ISet<int> animateClosing;
  @override
  final Size desktopSize;

  @override
  String toString() {
    return 'WindowStackState(stack: $stack, animateClosing: $animateClosing, desktopSize: $desktopSize)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$WindowStackStateImpl &&
            const DeepCollectionEquality().equals(other.stack, stack) &&
            const DeepCollectionEquality()
                .equals(other.animateClosing, animateClosing) &&
            (identical(other.desktopSize, desktopSize) ||
                other.desktopSize == desktopSize));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType,
      const DeepCollectionEquality().hash(stack),
      const DeepCollectionEquality().hash(animateClosing),
      desktopSize);

  /// Create a copy of WindowStackState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$WindowStackStateImplCopyWith<_$WindowStackStateImpl> get copyWith =>
      __$$WindowStackStateImplCopyWithImpl<_$WindowStackStateImpl>(
          this, _$identity);
}

abstract class _WindowStackState extends WindowStackState {
  const factory _WindowStackState(
      {required final IList<int> stack,
      required final ISet<int> animateClosing,
      required final Size desktopSize}) = _$WindowStackStateImpl;
  const _WindowStackState._() : super._();

  @override
  IList<int> get stack;
  @override
  ISet<int> get animateClosing;
  @override
  Size get desktopSize;

  /// Create a copy of WindowStackState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$WindowStackStateImplCopyWith<_$WindowStackStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
