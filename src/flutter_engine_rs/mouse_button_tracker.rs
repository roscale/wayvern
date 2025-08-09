use std::collections::HashSet;

use crate::flutter_engine_rs::flutter_engine_sys::{
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseForward,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle,
    FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary,
};

const BTN_LEFT: u32 = 0x110;
const BTN_RIGHT: u32 = 0x111;
const BTN_MIDDLE: u32 = 0x112;
const BTN_SIDE: u32 = 0x113;
const BTN_EXTRA: u32 = 0x114;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum FlutterMouseButton {
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
}

impl FlutterMouseButton {
    fn to_flutter_bit(&self) -> u32 {
        match self {
            FlutterMouseButton::Primary => {
                FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary
            }
            FlutterMouseButton::Secondary => {
                FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary
            }
            FlutterMouseButton::Middle => {
                FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle
            }
            FlutterMouseButton::Back => FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack,
            FlutterMouseButton::Forward => {
                FlutterPointerMouseButtons_kFlutterPointerButtonMouseForward
            }
        }
    }
}

#[derive(Default)]
pub struct FlutterMouseButtonTracker {
    down: HashSet<FlutterMouseButton>,
}

impl FlutterMouseButtonTracker {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn are_any_buttons_pressed(&self) -> bool {
        !self.down.is_empty()
    }

    pub fn press(&mut self, button: FlutterMouseButton) {
        self.down.insert(button);
    }

    pub fn release(&mut self, button: FlutterMouseButton) {
        self.down.remove(&button);
    }

    pub fn get_bitmask(&self) -> i64 {
        let mut flutter_mouse_buttons = 0;
        for button in self.down.iter() {
            flutter_mouse_buttons |= button.to_flutter_bit();
        }
        flutter_mouse_buttons as i64
    }
}

pub fn flutter_to_linux_button(button: u32) -> u32 {
    #[allow(non_upper_case_globals)]
    match button {
        FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary => BTN_LEFT,
        FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary => BTN_RIGHT,
        FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle => BTN_MIDDLE,
        FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack => BTN_SIDE,
        FlutterPointerMouseButtons_kFlutterPointerButtonMouseForward => BTN_EXTRA,
        _ => 0,
    }
}
