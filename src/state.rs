use smithay::backend::input::{ButtonState, KeyState, Keycode};
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::utils::SERIAL_COUNTER;
use crate::backends::Backend;
use crate::flutter_engine::KeyEvent;
use crate::server_state::Common;

pub struct State {
    pub common: Common,
    pub backend: Backend,
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

        let now_ms = self.common.now_ms();
        let tx = self.common.tx_flutter_handled_key_events.clone();

        self.common.flutter_engine.send_key_event(
            KeyEvent {
                key_code,
                specified_logical_key: None,
                codepoint: keysym.key_char(),
                state,
                time: now_ms,
                mods,
                mods_changed,
            },
            tx,
        ).unwrap();
    }
}
