use smithay::{
    delegate_compositor,
    reexports::wayland_server::{Client, protocol::wl_surface::WlSurface},
    wayland::{
        compositor::{
            BufferAssignment, CompositorClientState, CompositorHandler, CompositorState,
            SurfaceAttributes, with_states,
        },
        seat::WaylandFocus,
    },
};
use tracing::error;

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

        let Some(toplevel) = window.toplevel() else {
            return;
        };

        if !toplevel.is_initial_configure_sent() {
            toplevel.with_pending_state(|state| {
                state.size = self.common.initial_window_size;
            });
            toplevel.send_configure();
        }

        let has_buffer = with_states(surface, |data| {
            let mut guard = data.cached_state.get::<SurfaceAttributes>();
            let attributes = guard.current();
            matches!(attributes.buffer, Some(BufferAssignment::NewBuffer(_)))
        });

        let texture_id = 1234;

        self.common
            .surfaces_by_texture_id
            .insert(texture_id, surface.clone());

        if has_buffer {
            if let Err(err) = self
                .common
                .flutter
                .engine
                .register_external_texture(texture_id)
            {
                error!("Failed to register_external_texture({texture_id}): {err}");
                return;
            }

            if let Err(err) = self
                .common
                .flutter
                .engine
                .mark_external_texture_frame_available(texture_id)
            {
                error!("Failed to mark_external_texture_frame_available({texture_id}): {err}");
            }
        }
    }
}

delegate_compositor!(State);
