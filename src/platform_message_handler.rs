use smithay::backend::input::ButtonState;
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::reexports::calloop::channel::Channel;
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::calloop::LoopHandle;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::utils::SERIAL_COUNTER;
use platform_channels::encodable_value::EncodableValue;
use platform_channels::method_call::MethodCall;
use platform_channels::method_result::MethodResult;
use crate::mouse_button_tracker::FLUTTER_TO_LINUX_MOUSE_BUTTONS;
use crate::state::State;

macro_rules! extract {
    ($e:expr, $p:path) => {
        match $e {
            $p(value) => value,
            _ => panic!("Failed to extract value"),
        }
    };
}

fn get_value<'a>(map: &'a EncodableValue, key: &str) -> &'a EncodableValue {
    let map = extract!(map, EncodableValue::Map);
    for (k, v) in map {
        if let EncodableValue::String(k) = k {
            if k == key {
                return v;
            }
        }
    }
    panic!("Key {} not found in map", key);
}

pub fn register_platform_message_handler(
    loop_handle: &LoopHandle<'static, State>,
    rx_platform_message: Channel<(MethodCall, Box<dyn MethodResult>)>,
) {
    loop_handle
        .insert_source(
            rx_platform_message,
            |event, (), data| {
                let Msg((method_call, mut result)) = event else {
                    return;
                };

                match method_call.method() {
                    "pointer_hover" => {
                        let args = method_call.arguments().unwrap();
                        let view_id = get_value(args, "view_id").long_value().unwrap() as u64;
                        let x = *extract!(get_value(args, "x"), EncodableValue::Double);
                        let y = *extract!(get_value(args, "y"), EncodableValue::Double);

                        data.pointer_hover(view_id, x, y);

                        result.success(None);
                    }
                    "pointer_exit" => {
                        data.pointer_exit();

                        result.success(None);
                    }
                    "mouse_button_event" => {
                        let args = method_call.arguments().unwrap();
                        let button = get_value(args, "button").long_value().unwrap() as u32;
                        let is_pressed = *extract!(get_value(args, "is_pressed"), EncodableValue::Bool);

                        data.mouse_button_event(
                            *FLUTTER_TO_LINUX_MOUSE_BUTTONS.get(&(button)).unwrap() as u32,
                            is_pressed,
                        );

                        result.success(None);
                    }
                    "activate_window" => {
                        let args = method_call.arguments().unwrap();
                        let args = extract!(args, EncodableValue::List);
                        let view_id = args[0].long_value().unwrap() as u64;
                        let activate = extract!(args[1], EncodableValue::Bool);

                        data.activate_window(view_id, activate);

                        result.success(None);
                    }
                    "resize_window" => {
                        let args = method_call.arguments().unwrap();
                        let view_id = get_value(args, "view_id").long_value().unwrap() as u64;
                        let width = get_value(args, "width").long_value().unwrap() as i32;
                        let height = get_value(args, "height").long_value().unwrap() as i32;

                        data.resize_window(view_id, width, height);

                        result.success(None);
                    }
                    "unregister_view_texture" => {
                        let args = method_call.arguments().unwrap();
                        let texture_id = get_value(args, "texture_id").long_value().unwrap();

                        data.unregister_view_texture(texture_id);

                        result.success(None);
                    }
                    _ => {
                        result.success(None);
                    }
                }
            },
        )
        .expect("Failed to init wayland server source");
}

impl State {
    pub fn pointer_hover(&mut self, view_id: u64, x: f64, y: f64) {
        let pointer = self.common.pointer.clone();

        self.common.view_id_under_cursor = Some(view_id);

        let Some(surface) = self.common.surfaces.get(&view_id).cloned() else {
            return;
        };

        pointer.motion(
            self,
            Some((surface.clone(), (0.0, 0.0).into())),
            &MotionEvent {
                location: (x, y).into(),
                serial: SERIAL_COUNTER.next_serial(),
                time: self.common.now_ms(),
            },
        );
        pointer.frame(self);
    }

    pub fn pointer_exit(&mut self) {
        let pointer = self.common.pointer.clone();

        self.common.view_id_under_cursor = None;

        pointer.motion(
            self,
            None,
            &MotionEvent {
                location: (0.0, 0.0).into(),
                serial: SERIAL_COUNTER.next_serial(),
                time: self.common.now_ms(),
            },
        );
        pointer.frame(self);
    }

    pub fn mouse_button_event(&mut self, button: u32, is_pressed: bool) {
        let pointer = self.common.pointer.clone();

        pointer.button(
            self,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: self.common.now_ms(),
                button,
                state: if is_pressed { ButtonState::Pressed } else { ButtonState::Released },
            },
        );
        pointer.frame(self);
    }

    pub fn activate_window(&mut self, view_id: u64, activate: bool) {
        let pointer = self.common.seat.get_pointer().unwrap();
        let keyboard = self.common.seat.get_keyboard().unwrap();

        let serial = SERIAL_COUNTER.next_serial();

        if pointer.is_grabbed() {
            return;
        }

        let Some(toplevel) = self.common.xdg_toplevels.get(&view_id).cloned() else {
            return;
        };

        toplevel.with_pending_state(|state| {
            if activate {
                state.states.set(xdg_toplevel::State::Activated);
            } else {
                state.states.unset(xdg_toplevel::State::Activated);
            }
        });
        keyboard.set_focus(self, Some(toplevel.wl_surface().clone()), serial);

        for toplevel in self.common.xdg_toplevels.values() {
            toplevel.send_pending_configure();
        }
    }

    pub fn resize_window(&mut self, view_id: u64, width: i32, height: i32) {
        let Some(surface) = self.common.xdg_toplevels.get(&view_id).cloned() else {
            return;
        };

        surface.with_pending_state(|state| {
            state.size = Some((width, height).into());
        });
        surface.send_pending_configure();
    }
    
    pub fn unregister_view_texture(&mut self, texture_id: i64) {
        self.common.flutter_engine.unregister_external_texture(texture_id).unwrap();
    }
}