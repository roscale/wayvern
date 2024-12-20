// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of '../../../../ui/common/state/xdg_popup_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$XdgPopupState {
  int get parentViewId => throw _privateConstructorUsedError;
  Offset get position => throw _privateConstructorUsedError;
  GlobalKey<AnimationsState> get animationsKey =>
      throw _privateConstructorUsedError;
  bool get isClosing => throw _privateConstructorUsedError;

  /// Create a copy of XdgPopupState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $XdgPopupStateCopyWith<XdgPopupState> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $XdgPopupStateCopyWith<$Res> {
  factory $XdgPopupStateCopyWith(
          XdgPopupState value, $Res Function(XdgPopupState) then) =
      _$XdgPopupStateCopyWithImpl<$Res, XdgPopupState>;
  @useResult
  $Res call(
      {int parentViewId,
      Offset position,
      GlobalKey<AnimationsState> animationsKey,
      bool isClosing});
}

/// @nodoc
class _$XdgPopupStateCopyWithImpl<$Res, $Val extends XdgPopupState>
    implements $XdgPopupStateCopyWith<$Res> {
  _$XdgPopupStateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of XdgPopupState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? parentViewId = null,
    Object? position = null,
    Object? animationsKey = null,
    Object? isClosing = null,
  }) {
    return _then(_value.copyWith(
      parentViewId: null == parentViewId
          ? _value.parentViewId
          : parentViewId // ignore: cast_nullable_to_non_nullable
              as int,
      position: null == position
          ? _value.position
          : position // ignore: cast_nullable_to_non_nullable
              as Offset,
      animationsKey: null == animationsKey
          ? _value.animationsKey
          : animationsKey // ignore: cast_nullable_to_non_nullable
              as GlobalKey<AnimationsState>,
      isClosing: null == isClosing
          ? _value.isClosing
          : isClosing // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$XdgPopupStateImplCopyWith<$Res>
    implements $XdgPopupStateCopyWith<$Res> {
  factory _$$XdgPopupStateImplCopyWith(
          _$XdgPopupStateImpl value, $Res Function(_$XdgPopupStateImpl) then) =
      __$$XdgPopupStateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {int parentViewId,
      Offset position,
      GlobalKey<AnimationsState> animationsKey,
      bool isClosing});
}

/// @nodoc
class __$$XdgPopupStateImplCopyWithImpl<$Res>
    extends _$XdgPopupStateCopyWithImpl<$Res, _$XdgPopupStateImpl>
    implements _$$XdgPopupStateImplCopyWith<$Res> {
  __$$XdgPopupStateImplCopyWithImpl(
      _$XdgPopupStateImpl _value, $Res Function(_$XdgPopupStateImpl) _then)
      : super(_value, _then);

  /// Create a copy of XdgPopupState
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? parentViewId = null,
    Object? position = null,
    Object? animationsKey = null,
    Object? isClosing = null,
  }) {
    return _then(_$XdgPopupStateImpl(
      parentViewId: null == parentViewId
          ? _value.parentViewId
          : parentViewId // ignore: cast_nullable_to_non_nullable
              as int,
      position: null == position
          ? _value.position
          : position // ignore: cast_nullable_to_non_nullable
              as Offset,
      animationsKey: null == animationsKey
          ? _value.animationsKey
          : animationsKey // ignore: cast_nullable_to_non_nullable
              as GlobalKey<AnimationsState>,
      isClosing: null == isClosing
          ? _value.isClosing
          : isClosing // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc

class _$XdgPopupStateImpl
    with DiagnosticableTreeMixin
    implements _XdgPopupState {
  const _$XdgPopupStateImpl(
      {required this.parentViewId,
      required this.position,
      required this.animationsKey,
      required this.isClosing});

  @override
  final int parentViewId;
  @override
  final Offset position;
  @override
  final GlobalKey<AnimationsState> animationsKey;
  @override
  final bool isClosing;

  @override
  String toString({DiagnosticLevel minLevel = DiagnosticLevel.info}) {
    return 'XdgPopupState(parentViewId: $parentViewId, position: $position, animationsKey: $animationsKey, isClosing: $isClosing)';
  }

  @override
  void debugFillProperties(DiagnosticPropertiesBuilder properties) {
    super.debugFillProperties(properties);
    properties
      ..add(DiagnosticsProperty('type', 'XdgPopupState'))
      ..add(DiagnosticsProperty('parentViewId', parentViewId))
      ..add(DiagnosticsProperty('position', position))
      ..add(DiagnosticsProperty('animationsKey', animationsKey))
      ..add(DiagnosticsProperty('isClosing', isClosing));
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$XdgPopupStateImpl &&
            (identical(other.parentViewId, parentViewId) ||
                other.parentViewId == parentViewId) &&
            (identical(other.position, position) ||
                other.position == position) &&
            (identical(other.animationsKey, animationsKey) ||
                other.animationsKey == animationsKey) &&
            (identical(other.isClosing, isClosing) ||
                other.isClosing == isClosing));
  }

  @override
  int get hashCode => Object.hash(
      runtimeType, parentViewId, position, animationsKey, isClosing);

  /// Create a copy of XdgPopupState
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$XdgPopupStateImplCopyWith<_$XdgPopupStateImpl> get copyWith =>
      __$$XdgPopupStateImplCopyWithImpl<_$XdgPopupStateImpl>(this, _$identity);
}

abstract class _XdgPopupState implements XdgPopupState {
  const factory _XdgPopupState(
      {required final int parentViewId,
      required final Offset position,
      required final GlobalKey<AnimationsState> animationsKey,
      required final bool isClosing}) = _$XdgPopupStateImpl;

  @override
  int get parentViewId;
  @override
  Offset get position;
  @override
  GlobalKey<AnimationsState> get animationsKey;
  @override
  bool get isClosing;

  /// Create a copy of XdgPopupState
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$XdgPopupStateImplCopyWith<_$XdgPopupStateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
