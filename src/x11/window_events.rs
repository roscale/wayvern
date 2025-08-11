use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, ButtonState, InputBackend, InputEvent, MouseButton,
            PointerButtonEvent,
        },
        x11::{X11Event, X11Input},
    },
    output::Mode,
    utils::{Logical, Size},
};
use tracing::error;

use crate::{flutter_engine_rs::mouse_button_tracker::FlutterMouseButton, state::State};

pub fn handle_input_event(event: X11Event, state: &mut State) {
    match event {
        X11Event::CloseRequested { .. } => {
            state.common.should_stop = true;
        }
        X11Event::Resized { new_size, .. } => handle_resize_event(new_size, state),
        X11Event::Input { event, .. } => match event {
            InputEvent::PointerMotionAbsolute { event } => {
                handle_pointer_absolute_motion(event, state)
            }
            InputEvent::PointerButton { event } => handle_pointer_button(event, state),
            _ => {}
        },
        _ => {}
    }
}

pub fn handle_resize_event(new_size: Size<u16, Logical>, state: &mut State) {
    let x11 = state.backend.x11();

    for surface in x11.surfaces.iter() {
        if let Err(err) = state.common.flutter.engine.send_view_metrics(
            surface.view_id,
            new_size.w as usize,
            new_size.h as usize,
        ) {
            error!(
                "Failed to send metrics to view {}, w: {}, h: {}, {}",
                surface.view_id, new_size.w, new_size.h, err,
            );
        }

        let mode = Mode {
            size: (new_size.w as i32, new_size.h as i32).into(),
            refresh: 144,
        };
        state.common.output.set_preferred(mode);
        state
            .common
            .output
            .change_current_state(Some(mode), None, None, None);
    }
}

pub fn handle_pointer_absolute_motion(
    event: <X11Input as InputBackend>::PointerMotionAbsoluteEvent,
    state: &mut State,
) {
    // TODO(roscale): We should send an Add event to Flutter before sending the rest.
    state.backend.x11().mouse_position = (event.x(), event.y());
    let _ = state.common.flutter.engine.motion(event.x(), event.y());
}

pub fn handle_pointer_button(
    event: <X11Input as InputBackend>::PointerButtonEvent,
    state: &mut State,
) {
    let Some(button) = event.button() else {
        return;
    };

    let flutter_mouse_button = match button {
        MouseButton::Left => FlutterMouseButton::Primary,
        MouseButton::Middle => FlutterMouseButton::Middle,
        MouseButton::Right => FlutterMouseButton::Secondary,
        MouseButton::Back => FlutterMouseButton::Back,
        MouseButton::Forward => FlutterMouseButton::Forward,
        _ => return,
    };

    let pressed = match event.state() {
        ButtonState::Released => false,
        ButtonState::Pressed => true,
    };

    let mouse_position = state.backend.x11().mouse_position;

    let _ = state.common.flutter.engine.button(
        flutter_mouse_button,
        pressed,
        mouse_position.0,
        mouse_position.1,
    );
}
