use std::mem::size_of;

use crate::flutter_engine::FlutterEngine;
use embedder_sys::{FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse, FlutterPointerEvent, FlutterPointerPhase_kDown, FlutterPointerPhase_kHover, FlutterPointerPhase_kMove, FlutterPointerPhase_kUp, FlutterPointerSignalKind_kFlutterPointerSignalKindNone, FlutterPointerSignalKind_kFlutterPointerSignalKindScroll};
use input_linux::sys::KEY_ESC;
use smithay::backend::input;
use smithay::backend::input::{AbsolutePositionEvent, ButtonState, InputBackend, InputEvent, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent};
use crate::state::State;

pub fn handle_input(event: &InputEvent<impl InputBackend>, data: &mut State) {
    match event {
        InputEvent::DeviceAdded { .. } => {}
        InputEvent::DeviceRemoved { .. } => {}
        InputEvent::PointerMotion { event } => {
            data.common.mouse_position.0 += event.delta_x();
            data.common.mouse_position.1 += event.delta_y();
            send_motion_event(data);
        }
        InputEvent::PointerMotionAbsolute { event } => {
            data.common.mouse_position = (event.x(), event.y());
            send_motion_event(data);
        }
        InputEvent::PointerButton { event } => {
            let phase = if event.state() == ButtonState::Pressed {
                let are_any_buttons_pressed = data.common.flutter_engine.mouse_button_tracker.are_any_buttons_pressed();
                let _ = data.common.flutter_engine.mouse_button_tracker.press(event.button_code() as u16);
                if are_any_buttons_pressed {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                }
            } else {
                let _ = data.common.flutter_engine.mouse_button_tracker.release(event.button_code() as u16);
                if data.common.flutter_engine.mouse_button_tracker.are_any_buttons_pressed() {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kUp
                }
            };

            data.common.flutter_engine.send_pointer_event(FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>(),
                phase,
                timestamp: FlutterEngine::current_time_us() as usize,
                x: data.common.mouse_position.0,
                y: data.common.mouse_position.1,
                device: 0,
                signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
                scroll_delta_x: 0.0,
                scroll_delta_y: 0.0,
                device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
                buttons: data.common.flutter_engine.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::PointerAxis { event } => {
            data.common.flutter_engine.send_pointer_event(FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>(),
                phase: if data.common.flutter_engine.mouse_button_tracker.are_any_buttons_pressed() {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                },
                timestamp: FlutterEngine::current_time_us() as usize,
                x: data.common.mouse_position.0,
                y: data.common.mouse_position.1,
                device: 0,
                signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindScroll,
                scroll_delta_x: event.amount_v120(input::Axis::Horizontal).unwrap_or(0.0) * 10.0,
                scroll_delta_y: event.amount_v120(input::Axis::Vertical).unwrap_or(0.0) * 10.0,
                device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
                buttons: data.common.flutter_engine.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::Keyboard { event } => {
            data.handle_key_event(event.key_code(), event.state());

            if event.state() != KeyState::Pressed {
                return;
            }

            if event.key_code().raw() == KEY_ESC as u32 {
                data.common.should_stop = true;
            }
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
        InputEvent::SwitchToggle { .. } => {}
    }
}

fn send_motion_event(data: &mut State) {
    data.common.flutter_engine.send_pointer_event(FlutterPointerEvent {
        struct_size: size_of::<FlutterPointerEvent>(),
        phase: if data.common.flutter_engine.mouse_button_tracker.are_any_buttons_pressed() {
            FlutterPointerPhase_kMove
        } else {
            FlutterPointerPhase_kHover
        },
        timestamp: FlutterEngine::current_time_us() as usize,
        x: data.common.mouse_position.0,
        y: data.common.mouse_position.1,
        device: 0,
        signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
        scroll_delta_x: 0.0,
        scroll_delta_y: 0.0,
        device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
        buttons: data.common.flutter_engine.mouse_button_tracker.get_flutter_button_bitmask(),
        pan_x: 0.0,
        pan_y: 0.0,
        scale: 0.0,
        rotation: 0.0,
    }).unwrap();
}
