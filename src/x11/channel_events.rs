use std::{cell::RefCell, time::Duration};

use smithay::{
    backend::input::ButtonState,
    desktop::{WindowSurfaceType, space::SpaceElement},
    input::pointer::{ButtonEvent, MotionEvent},
    utils::{Logical, Point, SERIAL_COUNTER},
};
use tracing::error;

use crate::{
    flutter_engine_rs::{
        mouse_button_tracker::flutter_to_linux_button,
        platform_channels::{
            encodable_value::EncodableValue, method_call::MethodCall, method_result::MethodResult,
        },
    },
    state::State,
    wayland::handlers::{
        seat::{KeyboardFocusTarget, PointerFocusTarget},
        xdg::MyWindowData,
    },
};

pub fn handle_channel_events(
    method_call: MethodCall,
    mut result: Box<dyn MethodResult + 'static>,
    state: &mut State,
) {
    match method_call.method() {
        "setInitialWindowSize" => {
            handle_set_initial_window_size(&method_call, &mut result, state);
        }
        "motion" => {
            handle_motion(&method_call, &mut result, state);
        }
        "button" => {
            handle_button(method_call, &mut result, state);
        }

        _ => result.not_implemented(),
    }
}

fn handle_set_initial_window_size(
    method_call: &MethodCall,
    result: &mut Box<dyn MethodResult + 'static>,
    state: &mut State,
) {
    let arguments = method_call.arguments().unwrap();
    let EncodableValue::List(list) = arguments else {
        panic!();
    };

    let width = list[0].long_value().unwrap();
    let height = list[1].long_value().unwrap();

    state.common.initial_window_size = Some((width as i32, height as i32).into());

    result.success(None);
}

fn handle_motion(
    method_call: &MethodCall,
    result: &mut Box<dyn MethodResult + 'static>,
    state: &mut State,
) {
    let arguments = method_call.arguments().unwrap();
    let EncodableValue::List(list) = arguments else {
        panic!();
    };

    let surface_id = list[0].long_value().unwrap();
    let EncodableValue::Double(x) = list[1] else {
        panic!();
    };
    let EncodableValue::Double(y) = list[2] else {
        panic!();
    };

    let window = state
        .common
        .space
        .elements()
        .find(|window| {
            window
                .user_data()
                .get::<RefCell<MyWindowData>>()
                .unwrap()
                .borrow()
                .surface_id as i64
                == surface_id
        })
        .cloned();

    let Some(window) = window else {
        let message = format!("Window with surface id {surface_id} doesn't exist");
        error!(message);
        result.error("error".to_string(), message, None);
        return;
    };

    let point: Point<f64, Logical> = (x, y).into();

    if let Some((surface, location)) = window.surface_under(point, WindowSurfaceType::ALL) {
        let pointer = state.common.seat.get_pointer().unwrap();
        let location = point - location.to_f64();
        let time = Duration::from(state.common.clock.now()).as_millis() as u32;

        pointer.motion(
            state,
            Some((PointerFocusTarget::WlSurface(surface), (0.0, 0.0).into())),
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        pointer.frame(state);
    }

    let keyboard = state.common.seat.get_keyboard().unwrap();
    keyboard.set_focus(
        state,
        Some(KeyboardFocusTarget::Window(window)),
        SERIAL_COUNTER.next_serial(),
    );

    result.success(None);
}

fn handle_button(
    method_call: MethodCall,
    result: &mut Box<dyn MethodResult + 'static>,
    state: &mut State,
) {
    let arguments = method_call.arguments().unwrap();
    let EncodableValue::List(list) = arguments else {
        panic!();
    };

    let button = list[0].long_value().unwrap() as u32;
    let EncodableValue::Bool(pressed) = list[1] else {
        panic!();
    };

    let pointer = state.common.seat.get_pointer().unwrap();
    let time = Duration::from(state.common.clock.now()).as_millis() as u32;

    pointer.button(
        state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time,
            button: flutter_to_linux_button(button),
            state: if pressed {
                ButtonState::Pressed
            } else {
                ButtonState::Released
            },
        },
    );
    pointer.frame(state);

    result.success(None);
}
