use smithay::{delegate_compositor, reexports::wayland_server::{protocol::wl_surface::WlSurface, Client}, wayland::compositor::{CompositorClientState, CompositorHandler, CompositorState}};

use crate::state::{ClientState, State};

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.common.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_client_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        
    }
}

delegate_compositor!(State);
