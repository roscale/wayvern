use std::cell::RefCell;
use smithay::delegate_xdg_shell;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::protocol::wl_seat;
use smithay::utils::Serial;
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState};
use crate::backends::Backend;
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
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        let view_id = with_states(_surface.wl_surface(), |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });
        self.xdg_popups.insert(view_id, _surface.clone());
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
