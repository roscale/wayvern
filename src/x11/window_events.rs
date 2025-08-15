use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
            KeyboardKeyEvent, MouseButton, PointerAxisEvent, PointerButtonEvent,
        },
        x11::{X11Event, X11Input},
    },
    input::{keyboard::keysyms, pointer::AxisFrame},
    output::Mode,
    utils::{Logical, SERIAL_COUNTER, Size},
};
use tracing::error;

use crate::{
    flutter_engine_rs::{
        KeyState, keyboard::modifiers::FlutterModifiers, mouse_button_tracker::FlutterMouseButton,
    },
    state::State,
};

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
            InputEvent::PointerAxis { event } => handle_pointer_axis(event, state),
            InputEvent::Keyboard { event } => handle_keyboard(event, state),
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

    let _ = state
        .common
        .flutter
        .engine
        .button(flutter_mouse_button, pressed);
}

pub fn handle_pointer_axis(event: <X11Input as InputBackend>::PointerAxisEvent, state: &mut State) {
    let scroll_factor = 1.0;

    let mut frame = AxisFrame::new(event.time_msec()).source(event.source());

    if let Some(horizontal_amount) = event.amount(Axis::Horizontal) {
        if horizontal_amount != 0.0 {
            frame = frame.value(Axis::Horizontal, scroll_factor * horizontal_amount);
        } else {
            frame = frame.stop(Axis::Horizontal);
        }
    }

    if let Some(discrete) = event.amount_v120(Axis::Horizontal) {
        frame = frame.v120(Axis::Horizontal, (discrete * scroll_factor).round() as i32);
    }

    if let Some(vertical_amount) = event.amount(Axis::Vertical) {
        if vertical_amount != 0.0 {
            frame = frame.value(Axis::Vertical, scroll_factor * vertical_amount);
        } else {
            frame = frame.stop(Axis::Vertical);
        }
    }

    if let Some(discrete) = event.amount_v120(Axis::Vertical) {
        frame = frame.v120(Axis::Vertical, (discrete * scroll_factor).round() as i32);
    }

    let (dx, dy) = if frame.axis != (0.0, 0.0) {
        frame.axis
    } else if let Some(v120) = frame.v120 {
        (v120.0 as f64, v120.1 as f64)
    } else {
        (0.0, 0.0)
    };

    let pointer = state.common.seat.get_pointer().unwrap();

    pointer.axis(state, frame);
    pointer.frame(state);

    let _ = state.common.flutter.engine.axis(dx, dy);
}

pub fn handle_keyboard(event: <X11Input as InputBackend>::KeyboardKeyEvent, state: &mut State) {
    let keyboard = state.common.seat.get_keyboard().unwrap();

    let ((mods, keysym), mods_changed) = keyboard.input_intercept(
        state,
        event.key_code(),
        event.state(),
        |state, mods, keysym| (*mods, keysym.modified_sym()),
    );

    let handled = state.common.flutter.engine.send_key(
        match event.state() {
            smithay::backend::input::KeyState::Released => KeyState::Up,
            smithay::backend::input::KeyState::Pressed => KeyState::Down,
        },
        event.key_code().raw(),
        keysym.raw(),
        keysym.key_char(),
        FlutterModifiers {
            ctrl: mods.ctrl,
            alt: mods.alt,
            shift: mods.shift,
            logo: mods.logo,
            caps_lock: mods.caps_lock,
            num_lock: mods.num_lock,
        },
        false,
    );

    let forward = match handled {
        Ok(true) => false,
        Ok(false) => true,
        Err(err) => {
            error!("{err}");
            true
        }
    };

    if forward {
        keyboard.input_forward(
            state,
            event.key_code(),
            event.state(),
            SERIAL_COUNTER.next_serial(),
            event.time_msec(),
            mods_changed,
        );
    }
}
