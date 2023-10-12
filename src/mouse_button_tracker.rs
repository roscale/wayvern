use std::collections::HashSet;

use crate::flutter_engine::embedder::{
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseForward,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle,
    FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary,
};

#[derive(Default)]
pub struct MouseButtonTracker {
    down: HashSet<input_linux::Key>,
}

impl MouseButtonTracker {
    pub fn is_down(&self, button: input_linux::Key) -> bool {
        self.down.contains(&button)
    }

    pub fn are_any_buttons_pressed(&self) -> bool {
        !self.down.is_empty()
    }

    pub fn press(&mut self, button_code: u16) -> Result<(), input_linux::RangeError> {
        let key = input_linux::Key::from_code(button_code)?;
        self.down.insert(key);
        Ok(())
    }

    pub fn release(&mut self, button_code: u16) -> Result<(), input_linux::RangeError> {
        let key = input_linux::Key::from_code(button_code)?;
        self.down.remove(&key);
        Ok(())
    }

    pub fn get_flutter_button_bitmask(&self) -> i64 {
        let mut flutter_mouse_buttons = 0;
        if self.is_down(input_linux::Key::ButtonLeft) {
            flutter_mouse_buttons |= FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary;
        }
        if self.is_down(input_linux::Key::ButtonRight) {
            flutter_mouse_buttons |= FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary;
        }
        if self.is_down(input_linux::Key::ButtonMiddle) {
            flutter_mouse_buttons |= FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle;
        }
        if self.is_down(input_linux::Key::ButtonBack) {
            flutter_mouse_buttons |= FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack;
        }
        if self.is_down(input_linux::Key::ButtonForward) {
            flutter_mouse_buttons |= FlutterPointerMouseButtons_kFlutterPointerButtonMouseForward;
        }
        flutter_mouse_buttons as i64
    }
}
