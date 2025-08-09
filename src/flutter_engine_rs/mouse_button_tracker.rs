use std::collections::HashSet;

use crate::flutter_engine_rs::flutter_engine_sys::{
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseForward,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle,
    FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary,
    FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary, FlutterPointerPhase,
    FlutterPointerPhase_kDown, FlutterPointerPhase_kMove, FlutterPointerPhase_kUp,
};

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
