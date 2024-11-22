use crate::protocols::compositor::MySurfaceState;
use crate::state::State;
use platform_channels::encodable_value::EncodableValue;
use platform_channels::standard_method_codec::StandardMethodCodec;
use smithay::delegate_xdg_shell;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
use smithay::utils::Serial;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgPopupSurfaceData, XdgShellHandler, XdgShellState, XdgToplevelSurfaceData};

delegate_xdg_shell!(State);

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.common.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });
        self.common.xdg_toplevels.insert(view_id, surface.clone());

        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });

        self.common.flutter_engine.invoke_method(
            StandardMethodCodec::new(),
            "platform",
            "new_toplevel",
            Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
            ]))),
            None,
        );
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        let (view_id, parent) = with_states(_surface.wl_surface(), |surface_data| {
            let my_surface_data = surface_data.data_map.get::<MySurfaceState>().unwrap().borrow();
            let xdg_popup_surface_data = surface_data.data_map.get::<XdgPopupSurfaceData>().unwrap().lock().unwrap();

            (my_surface_data.view_id, xdg_popup_surface_data.parent.clone().unwrap())
        });

        let parent_view_id = with_states(&parent, |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });

        self.common.xdg_popups.insert(view_id, _surface.clone());

        self.common.flutter_engine.invoke_method(
            StandardMethodCodec::new(),
            "platform",
            "new_popup",
            Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
                (EncodableValue::String("parent".to_string()), EncodableValue::Int64(parent_view_id as i64)),
            ]))),
            None,
        );
    }

    fn move_request(&mut self, surface: ToplevelSurface, _seat: WlSeat, _serial: Serial) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });

        self.common.flutter_engine.invoke_method(
            StandardMethodCodec::new(),
            "platform",
            "interactive_move",
            Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
            ]))),
            None,
        );
    }

    fn resize_request(&mut self, surface: ToplevelSurface, _seat: WlSeat, _serial: Serial, edges: ResizeEdge) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });

        self.common.flutter_engine.invoke_method(
            StandardMethodCodec::new(),
            "platform",
            "interactive_resize",
            Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
                (EncodableValue::String("edge".to_string()), EncodableValue::Int64(edges as i64)),
            ]))),
            None,
        );
    }


    fn grab(&mut self, _surface: PopupSurface, _seat: WlSeat, _serial: Serial) {
        // Handle popup grab here
    }

    fn reposition_request(&mut self, surface: PopupSurface, _positioner: PositionerState, token: u32) {
        surface.send_repositioned(token);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });
        self.common.xdg_toplevels.remove(&view_id);
    }

    fn app_id_changed(&mut self, surface: ToplevelSurface) {
        let app_id = with_states(surface.wl_surface(), |states| {
            let attributes = states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap();

            attributes.app_id.clone().unwrap()
        });

        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });

        self.common.flutter_engine.invoke_method(
            StandardMethodCodec::new(),
            "platform",
            "set_app_id",
            Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
                (EncodableValue::String("app_id".to_string()), EncodableValue::String(app_id)),
            ]))),
            None,
        );
    }

    fn title_changed(&mut self, surface: ToplevelSurface) {
        let title = with_states(surface.wl_surface(), |states| {
            let attributes = states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap();

            attributes.title.clone().unwrap()
        });

        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<MySurfaceState>().unwrap().borrow().view_id
        });

        self.common.flutter_engine.invoke_method(
            StandardMethodCodec::new(),
            "platform",
            "set_title",
            Some(Box::new(EncodableValue::Map(vec![
                (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
                (EncodableValue::String("title".to_string()), EncodableValue::String(title)),
            ]))),
            None,
        );
    }
}
