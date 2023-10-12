use std::mem::size_of;
use smithay::backend::input;
use smithay::backend::input::{AbsolutePositionEvent, ButtonState, InputEvent, PointerAxisEvent, PointerButtonEvent};
use smithay::backend::x11::X11Input;
use crate::CalloopData;
use crate::flutter_engine::embedder::{FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse, FlutterPointerEvent, FlutterPointerPhase_kDown, FlutterPointerPhase_kHover, FlutterPointerPhase_kMove, FlutterPointerPhase_kUp, FlutterPointerSignalKind_kFlutterPointerSignalKindNone, FlutterPointerSignalKind_kFlutterPointerSignalKindScroll};
use crate::flutter_engine::FlutterEngine;
use crate::x11_client::X11Data;

pub fn handle_input(event: &input::InputEvent<X11Input>, data: &mut CalloopData<X11Data>) {
    match event {
        InputEvent::DeviceAdded { .. } => {}
        InputEvent::DeviceRemoved { .. } => {}
        InputEvent::PointerMotion { .. } => {}
        InputEvent::PointerMotionAbsolute { event } => {
            data.state.mouse_position = (event.x(), event.y());

            data.state.flutter_engine.send_pointer_event(FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>(),
                phase: if data.state.mouse_button_tracker.are_any_buttons_pressed() {
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
                buttons: data.state.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::PointerButton { event } => {
            event.state();

            let phase = if event.state() == ButtonState::Pressed {
                let are_any_buttons_pressed = data.state.mouse_button_tracker.are_any_buttons_pressed();
                let _ = data.state.mouse_button_tracker.press(event.button_code() as u16);
                if are_any_buttons_pressed {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                }
            } else {
                let _ = data.state.mouse_button_tracker.release(event.button_code() as u16);
                if data.state.mouse_button_tracker.are_any_buttons_pressed() {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kUp
                }
            };

            data.state.flutter_engine.send_pointer_event(FlutterPointerEvent {
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
                buttons: data.state.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::PointerAxis { event } => {
            data.state.flutter_engine.send_pointer_event(FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>(),
                phase: if data.state.mouse_button_tracker.are_any_buttons_pressed() {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                },
                timestamp: FlutterEngine::current_time_ms() as usize,
                x: data.state.mouse_position.0,
                y: data.state.mouse_position.1,
                device: 0,
                signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindScroll,
                scroll_delta_x: event.amount_discrete(input::Axis::Horizontal).unwrap() * 10.0,
                scroll_delta_y: event.amount_discrete(input::Axis::Vertical).unwrap() * 10.0,
                device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
                buttons: data.state.mouse_button_tracker.get_flutter_button_bitmask(),
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 0.0,
                rotation: 0.0,
            }).unwrap();
        }
        InputEvent::Keyboard { .. } => {}
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
