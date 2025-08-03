use std::rc::Rc;

use smithay::{
    delegate_xdg_shell,
    desktop::Window,
    reexports::wayland_server::protocol::wl_seat::WlSeat,
    utils::Serial,
    wayland::{
        compositor::{self, SurfaceAttributes, with_states},
        seat::WaylandFocus,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
    },
};

use crate::{
    flutter_engine_rs::platform_channels::{
        encodable_value::EncodableValue, method_call::MethodCall, method_channel::MethodChannel,
        standard_method_codec::StandardMethodCodec,
    },
    state::State,
};

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.common.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window_id = {
            let id = self.common.next_window_id;
            self.common.next_window_id += 1;
            id
        };

        with_states(surface.wl_surface(), |states| {
            states
                .data_map
                .insert_if_missing(|| MyWindowData { window_id });
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
            "window_created",
            Some(Box::new(EncodableValue::Int64(window_id as i64))),
            None,
        );
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let window_id = with_states(surface.wl_surface(), |states| {
            states.data_map.get::<MyWindowData>().unwrap().window_id
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
            "window_destroyed",
            Some(Box::new(EncodableValue::Int64(window_id as i64))),
            None,
        );
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {}

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        todo!()
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        todo!()
    }
}

struct MyWindowData {
    pub window_id: usize,
}

delegate_xdg_shell!(State);
