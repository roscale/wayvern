use smithay::delegate_seat;
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::input::pointer::CursorImageStatus;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Resource;
use smithay::wayland::selection::data_device::set_data_device_focus;
use crate::backends::Backend;
use crate::server_state::ServerState;

delegate_seat!(@<BackendData: Backend + 'static> ServerState<BackendData>);

impl<BackendData: Backend> SeatHandler for ServerState<BackendData> {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<ServerState<BackendData>> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, target: Option<&WlSurface>) {
        let dh = &self.display_handle;
        let client = target.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client);
    }

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}
