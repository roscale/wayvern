use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use smithay::delegate_xdg_shell;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge;
use smithay::reexports::wayland_server::protocol::wl_seat;
use smithay::reexports::wayland_server::protocol::wl_seat::WlSeat;
use smithay::utils::Serial;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgPopupSurfaceData, XdgShellHandler, XdgShellState};
use crate::backends::Backend;
use crate::flutter_engine::platform_channels::encodable_value::EncodableValue;
use crate::flutter_engine::platform_channels::method_channel::MethodChannel;
use crate::flutter_engine::platform_channels::standard_method_codec::StandardMethodCodec;
use crate::server_state::{MySurfaceState, ServerState};

delegate_xdg_shell!(@<BackendData: Backend + 'static> ServerState<BackendData>);

impl<BackendData: Backend> XdgShellHandler for ServerState<BackendData> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });
        self.xdg_toplevels.insert(view_id, surface.clone());

        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("new_toplevel", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
        ]))), None);
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        let (view_id, parent) = with_states(_surface.wl_surface(), |surface_data| {
            let my_surface_data = surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow();
            let xdg_popup_surface_data = surface_data.data_map.get::<XdgPopupSurfaceData>().unwrap().lock().unwrap();

            (my_surface_data.view_id, xdg_popup_surface_data.parent.clone().unwrap())
        });

        let parent_view_id = with_states(&parent, |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });

        self.xdg_popups.insert(view_id, _surface.clone());

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("new_popup", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
            (EncodableValue::String("parent".to_string()), EncodableValue::Int64(parent_view_id as i64)),
        ]))), None);
    }

    fn move_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("interactive_move", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
        ]))), None);
    }

    fn resize_request(&mut self, surface: ToplevelSurface, seat: WlSeat, serial: Serial, edges: ResizeEdge) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("interactive_resize", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
            (EncodableValue::String("edge".to_string()), EncodableValue::Int64(edges as i64)),
        ]))), None);
    }


    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Handle popup grab here
    }

    fn reposition_request(&mut self, surface: PopupSurface, positioner: PositionerState, token: u32) {
        surface.send_repositioned(token);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let view_id = with_states(surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });
        self.xdg_toplevels.remove(&view_id);
    }


}
