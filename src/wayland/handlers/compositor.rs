use std::{cell::RefCell, rc::Rc};

use smithay::{
    backend::renderer::{ImportAll, Texture, buffer_type},
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

use crate::{
    flutter_engine_rs::platform_channels::{
        encodable_value::EncodableValue, method_call::MethodCall, method_channel::MethodChannel,
        standard_method_codec::StandardMethodCodec,
    },
    state::{ClientState, State},
    wayland::handlers::xdg::MyWindowData,
};

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

        let (surface_id, buffer) = with_states(surface, |data| {
            let id = data
                .data_map
                .get::<RefCell<MyWindowData>>()
                .unwrap()
                .borrow_mut()
                .window_id;

            let buffer = data
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .buffer
                .take();

            (id, buffer)
        });

        if let Some(BufferAssignment::NewBuffer(buffer)) = buffer {
            let texture = self
                .backend
                .x11()
                .renderer
                .import_buffer(&buffer, None, &[]);

            buffer.release();

            let Some(texture) = texture else {
                error!("Can't import buffer type {:?}", buffer_type(&buffer));
                return;
            };

            let texture = match texture {
                Ok(texture) => texture,
                Err(err) => {
                    error!("Can't import buffer: {err:?}");
                    return;
                }
            };

            let current_texture = self
                .common
                .current_texture_by_surface_id
                .get(&surface_id)
                .cloned();

            let (texture_id, is_new) = if let Some((id, current_texture)) = current_texture
                && texture.size() == current_texture.size()
            {
                (id, false)
            } else {
                let texture_id = self.common.next_texture_id;
                self.common.next_texture_id += 1;

                (texture_id, true)
            };

            self.common
                .textures_by_texture_id
                .insert(texture_id, texture.clone());

            self.common
                .current_texture_by_surface_id
                .insert(surface_id, (texture_id, texture.clone()));

            if is_new {
                self.common
                    .flutter
                    .engine
                    .register_external_texture(texture_id);
            }

            self.common
                .flutter
                .engine
                .mark_external_texture_frame_available(texture_id);

            MethodChannel::new(
                self.common.flutter.engine.binary_messenger(),
                "wayvern/window".to_string(),
                Rc::new(StandardMethodCodec::new()),
            )
            .invoke_method(
                "textureAttached",
                Some(Box::new(EncodableValue::Int64List(Vec::from([
                    surface_id as i64,
                    texture_id,
                    texture.width() as i64,
                    texture.height() as i64,
                ])))),
                None,
            );
        }
    }
}

delegate_compositor!(State);
