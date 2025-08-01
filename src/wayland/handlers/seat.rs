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
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{IsAlive, Serial},
    wayland::{seat::WaylandFocus, shell::xdg::PopupSurface},
};

use crate::state::State;

impl SeatHandler for State {
    type KeyboardFocus = KeyboardFocusTarget;

    type PointerFocus = PointerFocusTarget;

    type TouchFocus = TouchFocusTarget;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.common.seat_state
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
        _seat: &Seat<State>,
        _data: &mut State,
        _keys: Vec<KeysymHandle<'_>>,
        _serial: Serial,
    ) {
        todo!()
    }

    fn leave(&self, _seat: &Seat<State>, _data: &mut State, _serial: Serial) {
        todo!()
    }

    fn key(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _key: KeysymHandle<'_>,
        _state: KeyState,
        _serial: Serial,
        _time: u32,
    ) {
        todo!()
    }

    fn modifiers(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _modifiers: ModifiersState,
        _serial: Serial,
    ) {
        todo!()
    }
}

impl PointerTarget<State> for PointerFocusTarget {
    fn enter(&self, _seat: &Seat<State>, _data: &mut State, _event: &pointer::MotionEvent) {
        todo!()
    }

    fn motion(&self, _seat: &Seat<State>, _data: &mut State, _event: &pointer::MotionEvent) {
        todo!()
    }

    fn relative_motion(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::RelativeMotionEvent,
    ) {
        todo!()
    }

    fn button(&self, _seat: &Seat<State>, _data: &mut State, _event: &pointer::ButtonEvent) {
        todo!()
    }

    fn axis(&self, _seat: &Seat<State>, _data: &mut State, _frame: pointer::AxisFrame) {
        todo!()
    }

    fn frame(&self, _seat: &Seat<State>, _data: &mut State) {
        todo!()
    }

    fn gesture_swipe_begin(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GestureSwipeBeginEvent,
    ) {
        todo!()
    }

    fn gesture_swipe_update(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GestureSwipeUpdateEvent,
    ) {
        todo!()
    }

    fn gesture_swipe_end(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GestureSwipeEndEvent,
    ) {
        todo!()
    }

    fn gesture_pinch_begin(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GesturePinchBeginEvent,
    ) {
        todo!()
    }

    fn gesture_pinch_update(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GesturePinchUpdateEvent,
    ) {
        todo!()
    }

    fn gesture_pinch_end(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GesturePinchEndEvent,
    ) {
        todo!()
    }

    fn gesture_hold_begin(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GestureHoldBeginEvent,
    ) {
        todo!()
    }

    fn gesture_hold_end(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &pointer::GestureHoldEndEvent,
    ) {
        todo!()
    }

    fn leave(&self, _seat: &Seat<State>, _data: &mut State, _serial: Serial, _time: u32) {
        todo!()
    }
}

impl TouchTarget<State> for TouchFocusTarget {
    fn down(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &touch::DownEvent,
        _seq: Serial,
    ) {
        todo!()
    }

    fn up(&self, _seat: &Seat<State>, _data: &mut State, _event: &touch::UpEvent, _seq: Serial) {
        todo!()
    }

    fn motion(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &touch::MotionEvent,
        _seq: Serial,
    ) {
        todo!()
    }

    fn frame(&self, _seat: &Seat<State>, _data: &mut State, _seq: Serial) {
        todo!()
    }

    fn cancel(&self, _seat: &Seat<State>, _data: &mut State, _seq: Serial) {
        todo!()
    }

    fn shape(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &touch::ShapeEvent,
        _seq: Serial,
    ) {
        todo!()
    }

    fn orientation(
        &self,
        _seat: &Seat<State>,
        _data: &mut State,
        _event: &touch::OrientationEvent,
        _seq: Serial,
    ) {
        todo!()
    }
}

delegate_seat!(State);
