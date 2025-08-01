use smithay::{
    backend::renderer::utils::on_commit_buffer_handler, delegate_compositor, reexports::wayland_server::{protocol::wl_surface::WlSurface, Client}, utils::Size, wayland::{
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        seat::WaylandFocus,
    }
};

use crate::state::{ClientState, State};

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.common.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client
            .get_data::<ClientState>()
            .unwrap()
            .compositor_client_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        let window = self
            .common
            .windows
            .iter()
            .find(|w| w.wl_surface().unwrap().as_ref() == surface);

        let Some(window) = window else {
            return;
        };

        if let Some(toplevel) = window.toplevel()
            && !toplevel.is_initial_configure_sent()
        {
            toplevel.with_pending_state(|state| {
                state.size = self.common.initial_window_size;
            });
            toplevel.send_configure();
        }
    }
}

delegate_compositor!(State);
