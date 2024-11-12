use std::mem::size_of;

use crate::flutter_engine::{FlutterEngine, KeyEvent};
use embedder_sys::{FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse, FlutterPointerEvent, FlutterPointerPhase_kDown, FlutterPointerPhase_kHover, FlutterPointerPhase_kMove, FlutterPointerPhase_kUp, FlutterPointerSignalKind_kFlutterPointerSignalKindNone, FlutterPointerSignalKind_kFlutterPointerSignalKindScroll};
use input_linux::sys::KEY_ESC;
use smithay::backend::input;
use smithay::backend::input::{AbsolutePositionEvent, ButtonState, InputBackend, InputEvent, KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent, PointerMotionEvent};
use smithay::input::keyboard::Keycode;
use smithay::reexports::calloop::channel::Channel;
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::calloop::LoopHandle;
use smithay::utils::SERIAL_COUNTER;
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

impl State {
    pub fn handle_key_event(&mut self, key_code: Keycode, state: KeyState) {
        let keyboard = self.common.keyboard.clone();

        print!("Key event: {:?} {:?}", key_code, state);
        if state == KeyState::Pressed && key_code.raw() as i32 == 9 {
            self.common.should_stop = true;
            return;
        }

        // Update the keyboard state without forwarding the event to the client.
        let ((mods, keysym), mods_changed) =
            keyboard.input_intercept::<_, _>(self, key_code, state, |_, mods, keysym_handle| {
                (*mods, keysym_handle.modified_sym())
            });

        self.common.flutter_engine.send_key_event(
            KeyEvent {
                key_code,
                specified_logical_key: None,
                codepoint: keysym.key_char(),
                state,
                time: self.common.now_ms(),
                mods,
                mods_changed,
            },
            self.common.tx_flutter_handled_key_event.clone(),
        ).unwrap();
    }
}

pub fn register_flutter_handled_key_event_handler(
    loop_handle: &LoopHandle<State>,
    rx_flutter_handled_key_event: Channel<(KeyEvent, bool)>,
) {
    loop_handle.insert_source(rx_flutter_handled_key_event, |event, (), mut data| {
        let Msg((key_event, handled)) = event else {
            return;
        };

        if handled {
            // Flutter consumed this event. Probably a keyboard shortcut.
            return;
        }

        let text_input = &mut data.common.flutter_engine.text_input;

        if text_input.is_active() {
            if key_event.state == KeyState::Pressed
                && !key_event.mods.ctrl
                && !key_event.mods.alt
            {
                // text_input.press_key(key_event.key_code.raw(), key_event.codepoint);
            }
            // It doesn't matter if the text field captured the key event or not.
            // As long as it stays active, don't forward events to the Wayland client.
            return;
        }

        // The compositor was not interested in this event,
        // so we forward it to the Wayland client in focus
        // if there is one.
        let keyboard = data.common.keyboard.clone();
        keyboard.input_forward(
            &mut data,
            key_event.key_code,
            key_event.state,
            SERIAL_COUNTER.next_serial(),
            key_event.time,
            key_event.mods_changed,
        );
    }).unwrap();
}
