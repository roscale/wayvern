use std::{cell::RefCell, rc::Rc};

use smithay::{
    backend::renderer::{gles::GlesTexture, Texture},
    delegate_xdg_shell,
    desktop::Window,
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

        self.common
            .windows
            .push(Window::new_wayland_window(surface));

        let mut channel = MethodChannel::new(
            self.common.flutter.engine.binary_messenger(),
            "wayvern/window".to_string(),
            Rc::new(StandardMethodCodec::new()),
        );

        channel.invoke_method(
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

        self.common
            .windows
            .retain(|w| w.wl_surface().is_none_or(|s| &*s != surface.wl_surface()));

        let mut channel = MethodChannel::new(
            self.common.flutter.engine.binary_messenger(),
            "wayvern/window".to_string(),
            Rc::new(StandardMethodCodec::new()),
        );

        channel.invoke_method(
            "windowDestroyed",
            Some(Box::new(EncodableValue::Int64(window_id as i64))),
            None,
        );
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

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
