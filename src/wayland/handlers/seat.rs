use std::borrow::Cow;

use smithay::{
    backend::input::KeyState,
    delegate_seat,
    desktop::Window,
    input::{
        Seat, SeatHandler, SeatState,
        keyboard::{KeyboardTarget, KeysymHandle, ModifiersState},
        pointer::{self, PointerTarget},
        touch::{self, TouchTarget},
    },
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{IsAlive, Serial},
    wayland::{
        seat::WaylandFocus, selection::data_device::set_data_device_focus, shell::xdg::PopupSurface,
    },
};

use crate::state::State;

impl SeatHandler for State {
    type KeyboardFocus = KeyboardFocusTarget;

    type PointerFocus = PointerFocusTarget;

    type TouchFocus = TouchFocusTarget;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.common.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&KeyboardFocusTarget>) {
        let display_handle = &self.common.display_handle;
        let client =
            focused.and_then(|s| display_handle.get_client(s.wl_surface().unwrap().id()).ok());
        set_data_device_focus(display_handle, seat, client);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyboardFocusTarget {
    Window(Window),
    Popup(PopupSurface),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PointerFocusTarget {
    WlSurface(WlSurface),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TouchFocusTarget {
    WlSurface(WlSurface),
}

impl IsAlive for KeyboardFocusTarget {
    fn alive(&self) -> bool {
        match self {
            KeyboardFocusTarget::Window(window) => window.alive(),
            KeyboardFocusTarget::Popup(popup_surface) => popup_surface.alive(),
        }
    }
}

impl IsAlive for PointerFocusTarget {
    fn alive(&self) -> bool {
        match self {
            PointerFocusTarget::WlSurface(wl_surface) => wl_surface.alive(),
        }
    }
}

impl IsAlive for TouchFocusTarget {
    fn alive(&self) -> bool {
        match self {
            TouchFocusTarget::WlSurface(wl_surface) => wl_surface.alive(),
        }
    }
}

impl WaylandFocus for KeyboardFocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        match self {
            KeyboardFocusTarget::Window(window) => window.wl_surface(),
            KeyboardFocusTarget::Popup(popup_surface) => {
                Some(Cow::Borrowed(popup_surface.wl_surface()))
            }
        }
    }
}

impl WaylandFocus for PointerFocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        match self {
            PointerFocusTarget::WlSurface(wl_surface) => Some(Cow::Borrowed(wl_surface)),
        }
    }
}

impl WaylandFocus for TouchFocusTarget {
    fn wl_surface(&self) -> Option<Cow<'_, WlSurface>> {
        match self {
            TouchFocusTarget::WlSurface(wl_surface) => Some(Cow::Borrowed(wl_surface)),
        }
    }
}

impl KeyboardTarget<State> for KeyboardFocusTarget {
    fn enter(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        _keys: Vec<KeysymHandle<'_>>,
        _serial: Serial,
    ) {
        todo!()
    }

    fn leave(&self, seat: &Seat<State>, data: &mut State, _serial: Serial) {
        todo!()
    }

    fn key(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        _key: KeysymHandle<'_>,
        _state: KeyState,
        _serial: Serial,
        _time: u32,
    ) {
        todo!()
    }

    fn modifiers(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        _modifiers: ModifiersState,
        _serial: Serial,
    ) {
        todo!()
    }
}

impl PointerTarget<State> for PointerFocusTarget {
    fn enter(&self, seat: &Seat<State>, data: &mut State, event: &pointer::MotionEvent) {
        match self {
            PointerFocusTarget::WlSurface(s) => PointerTarget::enter(s, seat, data, event),
        }
    }

    fn motion(&self, seat: &Seat<State>, data: &mut State, event: &pointer::MotionEvent) {
        match self {
            PointerFocusTarget::WlSurface(s) => PointerTarget::motion(s, seat, data, event),
        }
    }

    fn relative_motion(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::RelativeMotionEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::relative_motion(s, seat, data, event)
            }
        }
    }

    fn button(&self, seat: &Seat<State>, data: &mut State, event: &pointer::ButtonEvent) {
        match self {
            PointerFocusTarget::WlSurface(s) => PointerTarget::button(s, seat, data, event),
        }
    }

    fn axis(&self, seat: &Seat<State>, data: &mut State, frame: pointer::AxisFrame) {
        match self {
            PointerFocusTarget::WlSurface(s) => PointerTarget::axis(s, seat, data, frame),
        }
    }

    fn frame(&self, seat: &Seat<State>, data: &mut State) {
        match self {
            PointerFocusTarget::WlSurface(s) => PointerTarget::frame(s, seat, data),
        }
    }

    fn gesture_swipe_begin(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GestureSwipeBeginEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_swipe_begin(s, seat, data, event)
            }
        }
    }

    fn gesture_swipe_update(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GestureSwipeUpdateEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_swipe_update(s, seat, data, event)
            }
        }
    }

    fn gesture_swipe_end(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GestureSwipeEndEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_swipe_end(s, seat, data, event)
            }
        }
    }

    fn gesture_pinch_begin(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GesturePinchBeginEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_pinch_begin(s, seat, data, event)
            }
        }
    }

    fn gesture_pinch_update(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GesturePinchUpdateEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_pinch_update(s, seat, data, event)
            }
        }
    }

    fn gesture_pinch_end(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GesturePinchEndEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_pinch_end(s, seat, data, event)
            }
        }
    }

    fn gesture_hold_begin(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GestureHoldBeginEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_hold_begin(s, seat, data, event)
            }
        }
    }

    fn gesture_hold_end(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &pointer::GestureHoldEndEvent,
    ) {
        match self {
            PointerFocusTarget::WlSurface(s) => {
                PointerTarget::gesture_hold_end(s, seat, data, event)
            }
        }
    }

    fn leave(&self, seat: &Seat<State>, data: &mut State, serial: Serial, time: u32) {
        match self {
            PointerFocusTarget::WlSurface(s) => PointerTarget::leave(s, seat, data, serial, time),
        }
    }
}

impl TouchTarget<State> for TouchFocusTarget {
    fn down(&self, seat: &Seat<State>, data: &mut State, event: &touch::DownEvent, _seq: Serial) {
        todo!()
    }

    fn up(&self, seat: &Seat<State>, data: &mut State, event: &touch::UpEvent, _seq: Serial) {
        todo!()
    }

    fn motion(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &touch::MotionEvent,
        _seq: Serial,
    ) {
        todo!()
    }

    fn frame(&self, seat: &Seat<State>, data: &mut State, _seq: Serial) {
        todo!()
    }

    fn cancel(&self, seat: &Seat<State>, data: &mut State, _seq: Serial) {
        todo!()
    }

    fn shape(&self, seat: &Seat<State>, data: &mut State, event: &touch::ShapeEvent, _seq: Serial) {
        todo!()
    }

    fn orientation(
        &self,
        seat: &Seat<State>,
        data: &mut State,
        event: &touch::OrientationEvent,
        _seq: Serial,
    ) {
        todo!()
    }
}

delegate_seat!(State);
