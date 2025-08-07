use std::{cell::RefCell, rc::Rc};

use smithay::{
    backend::renderer::{Texture, gles::GlesTexture},
    delegate_xdg_shell,
    desktop::{PopupKind, Window},
    reexports::wayland_server::protocol::wl_seat::WlSeat,
    utils::Serial,
    wayland::{
        compositor::with_states,
        seat::WaylandFocus,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
    },
};
use tracing::error;

use crate::{
    flutter_engine_rs::platform_channels::{
        encodable_value::EncodableValue, method_channel::MethodChannel,
        standard_method_codec::StandardMethodCodec,
    },
    state::State,
};

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.common.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window_id = self.common.next_window_id;
        self.common.next_window_id += 1;

        with_states(surface.wl_surface(), |states| {
            states.data_map.insert_if_missing(|| {
                RefCell::new(MyWindowData {
                    window_id,
                    current_texture: None,
                    old_texture: None,
                })
            });
        });

        let window = Window::new_wayland_window(surface);

        self.common.space.map_element(window, (0, 0), true);

        MethodChannel::new(
            self.common.flutter.engine.binary_messenger(),
            "wayvern/window".to_string(),
            Rc::new(StandardMethodCodec::new()),
        )
        .invoke_method(
            "windowCreated",
            Some(Box::new(EncodableValue::Int64(window_id as i64))),
            None,
        );
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let window_id = with_states(surface.wl_surface(), |states| {
            states
                .data_map
                .get::<RefCell<MyWindowData>>()
                .unwrap()
                .borrow_mut()
                .window_id
        });

        let window = self
            .common
            .space
            .elements()
            .find(|e| matches!(e.wl_surface(), Some(s) if &*s == surface.wl_surface()))
            .cloned();

        if let Some(window) = window {
            self.common.space.unmap_elem(&window);
        }

        MethodChannel::new(
            self.common.flutter.engine.binary_messenger(),
            "wayvern/window".to_string(),
            Rc::new(StandardMethodCodec::new()),
        )
        .invoke_method(
            "windowDestroyed",
            Some(Box::new(EncodableValue::Int64(window_id as i64))),
            None,
        );
    }

    fn new_popup(&mut self, surface: PopupSurface, _positioner: PositionerState) {
        if let Err(err) = self.common.popups.track_popup(PopupKind::from(surface)) {
            error!("Failed to track popup: {err}");
        }
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        todo!()
    }

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        todo!()
    }
}

pub struct MyWindowData {
    pub window_id: usize,
    pub current_texture: Option<GlesTexture>,
    pub old_texture: Option<GlesTexture>,
}

delegate_xdg_shell!(State);
