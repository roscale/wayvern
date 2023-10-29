use std::mem::size_of;
use std::rc::Rc;
use std::sync::atomic::Ordering;

use input_linux::sys::KEY_ESC;
use smithay::backend::input;
use smithay::backend::input::{AbsolutePositionEvent, ButtonState, InputBackend, InputEvent, KeyboardKeyEvent, KeyState, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent};

use crate::{Backend, CalloopData};
use crate::flutter_engine::embedder::{FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse, FlutterPointerEvent, FlutterPointerPhase_kDown, FlutterPointerPhase_kHover, FlutterPointerPhase_kMove, FlutterPointerPhase_kUp, FlutterPointerSignalKind_kFlutterPointerSignalKindNone, FlutterPointerSignalKind_kFlutterPointerSignalKindScroll};
use crate::flutter_engine::FlutterEngine;
use crate::flutter_engine::platform_channels::binary_messenger_impl::BinaryMessengerImpl;
use crate::flutter_engine::platform_channels::encodable_value::EncodableValue;
use crate::flutter_engine::platform_channels::method_channel::MethodChannel;
use crate::flutter_engine::platform_channels::standard_method_codec::StandardMethodCodec;

pub fn handle_input(event: &InputEvent<impl InputBackend>, data: &mut CalloopData<impl Backend>) {
    match event {
        InputEvent::DeviceAdded { .. } => {}
        InputEvent::DeviceRemoved { .. } => {}
        InputEvent::PointerMotion { event } => {
            data.state.mouse_position.0 += event.delta_x();
            data.state.mouse_position.1 += event.delta_y();
            send_motion_event(data);
        }
        InputEvent::PointerMotionAbsolute { event } => {
            data.state.mouse_position = (event.x(), event.y());
            send_motion_event(data);
        }
        InputEvent::PointerButton { event } => {
            event.state();

            let phase = if event.state() == ButtonState::Pressed {
                let are_any_buttons_pressed = data.flutter_engine.mouse_button_tracker.are_any_buttons_pressed();
                let _ = data.flutter_engine.mouse_button_tracker.press(event.button_code() as u16);
                if are_any_buttons_pressed {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                }
            } else {
                let _ = data.flutter_engine.mouse_button_tracker.release(event.button_code() as u16);
                if data.flutter_engine.mouse_button_tracker.are_any_buttons_pressed() {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kUp
                }
            };

            data.flutter_engine.send_pointer_event(FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>(),
                phase,
                timestamp: FlutterEngine::current_time_ms() as usize,
                x: data.state.mouse_position.0,
                y: data.state.mouse_position.1,
                device: 0,
                signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
                scroll_delta_x: 0.0,
                scroll_delta_y: 0.0,
                device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
                buttons: data.flutter_engine.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::PointerAxis { event } => {
            data.flutter_engine.send_pointer_event(FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>(),
                phase: if data.flutter_engine.mouse_button_tracker.are_any_buttons_pressed() {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                },
                timestamp: FlutterEngine::current_time_ms() as usize,
                x: data.state.mouse_position.0,
                y: data.state.mouse_position.1,
                device: 0,
                signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindScroll,
                scroll_delta_x: event.amount_discrete(input::Axis::Horizontal).unwrap_or(0.0) * 10.0,
                scroll_delta_y: event.amount_discrete(input::Axis::Vertical).unwrap_or(0.0) * 10.0,
                device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
                buttons: data.flutter_engine.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::Keyboard { event } => {
            if event.state() != KeyState::Pressed {
                return;
            }

            if event.key_code() == KEY_ESC as u32 {
                data.state.running.store(false, Ordering::SeqCst);
                return;
            }

            let mut messenger = BinaryMessengerImpl::new(data.flutter_engine.handle);
            let codec = Rc::new(StandardMethodCodec::new());
            let mut method_channel = MethodChannel::new(&mut messenger, "test_channel".to_string(), codec);
            method_channel.invoke_method("test", Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::Int32(3), EncodableValue::String("three".to_string())),
                (EncodableValue::Bool(false), EncodableValue::List(vec![
                    EncodableValue::Int32(1),
                    EncodableValue::Int32(2),
                    EncodableValue::Int32(3),
                ])),
            ]))), None);
        }
        InputEvent::GestureSwipeBegin { .. } => {}
        InputEvent::GestureSwipeUpdate { .. } => {}
        InputEvent::GestureSwipeEnd { .. } => {}
        InputEvent::GesturePinchBegin { .. } => {}
        InputEvent::GesturePinchUpdate { .. } => {}
        InputEvent::GesturePinchEnd { .. } => {}
        InputEvent::GestureHoldBegin { .. } => {}
        InputEvent::GestureHoldEnd { .. } => {}
        InputEvent::TouchDown { .. } => {}
        InputEvent::TouchMotion { .. } => {}
        InputEvent::TouchUp { .. } => {}
        InputEvent::TouchCancel { .. } => {}
        InputEvent::TouchFrame { .. } => {}
        InputEvent::TabletToolAxis { .. } => {}
        InputEvent::TabletToolProximity { .. } => {}
        InputEvent::TabletToolTip { .. } => {}
        InputEvent::TabletToolButton { .. } => {}
        InputEvent::Special(_) => {}
    }
}

fn send_motion_event(data: &mut CalloopData<impl Backend>) {
    data.flutter_engine.send_pointer_event(FlutterPointerEvent {
        struct_size: size_of::<FlutterPointerEvent>(),
        phase: if data.flutter_engine.mouse_button_tracker.are_any_buttons_pressed() {
            FlutterPointerPhase_kMove
        } else {
            FlutterPointerPhase_kHover
        },
        timestamp: FlutterEngine::current_time_ms() as usize,
        x: data.state.mouse_position.0,
        y: data.state.mouse_position.1,
        device: 0,
        signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
        scroll_delta_x: 0.0,
        scroll_delta_y: 0.0,
        device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
        buttons: data.flutter_engine.mouse_button_tracker.get_flutter_button_bitmask(),
        pan_x: 0.0,
        pan_y: 0.0,
        scale: 0.0,
        rotation: 0.0,
    }).unwrap();
}
