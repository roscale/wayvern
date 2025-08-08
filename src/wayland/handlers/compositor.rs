use std::{cell::RefCell, rc::Rc};

use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, ImportAll, Offscreen, Renderer, Texture, buffer_type,
            element::{
                AsRenderElements, Element, Kind,
                surface::{WaylandSurfaceRenderElement, render_elements_from_surface_tree},
            },
            gles::{GlesRenderbuffer, GlesRenderer, GlesTarget, GlesTexture},
            utils::{draw_render_elements, on_commit_buffer_handler},
        },
    },
    delegate_compositor,
    desktop::{
        PopupKind, PopupManager, Space, Window, WindowSurface, utils::with_surfaces_surface_tree,
    },
    reexports::wayland_server::{Client, protocol::wl_surface::WlSurface},
    utils::{Buffer, Logical, Rectangle, Scale, Size, Transform},
    wayland::{
        compositor::{
            BufferAssignment, CompositorClientState, CompositorHandler, CompositorState,
            SurfaceAttributes, get_parent, is_sync_subsurface, with_states,
        },
        seat::WaylandFocus,
    },
};
use tracing::error;

use crate::{
    flutter_engine_rs::platform_channels::{
        encodable_value::EncodableValue, method_channel::MethodChannel,
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
        on_commit_buffer_handler::<Self>(surface);
        self.common.popups.commit(surface);

        ensure_initial_configure(
            surface,
            self.common.initial_window_size,
            &self.common.space,
            &mut self.common.popups,
        );

        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .common
                .space
                .elements()
                .find(|w| w.wl_surface().is_some_and(|s| *s == root))
            {
                window.on_commit();
            }
        };

        let Some(window) = self
            .common
            .space
            .elements()
            .find(|window| window.wl_surface().is_some_and(|s| &*s == surface))
            .cloned()
        else {
            return;
        };

        let output_scale = self.common.output.current_scale();

        let renderer = &mut self.backend.x11().renderer;
        let render_elements = render_window(&window, renderer, output_scale.fractional_scale());

        if render_elements.is_empty() {
            return;
        }

        // Get the surface with the biggest buffer scale.
        let mut max_scale = 0;
        with_surfaces_surface_tree(surface, |_, data| {
            max_scale = max_scale.max(
                data.cached_state
                    .get::<SurfaceAttributes>()
                    .current()
                    .buffer_scale,
            );
        });

        let logical_size = window.bbox().size;
        let buffer_size = logical_size.to_buffer(max_scale, Transform::Normal);
        let physical_size = logical_size.to_physical(output_scale.integer_scale());

        let mut texture: GlesTexture = renderer
            .create_buffer(Fourcc::Argb8888, buffer_size)
            .unwrap();

        {
            let mut target = renderer.bind(&mut texture).unwrap();

            let mut frame = renderer
                .render(&mut target, physical_size, Transform::Normal)
                .unwrap();

            draw_render_elements(
                &mut frame,
                output_scale.fractional_scale(),
                &render_elements,
                &[Rectangle::from_size(physical_size)],
            )
            .unwrap();
        }

        let surface_id = with_states(surface, |data| {
            data.data_map
                .get::<RefCell<MyWindowData>>()
                .unwrap()
                .borrow_mut()
                .window_id
        });

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

        if is_new
            && let Err(err) = self
                .common
                .flutter
                .engine
                .register_external_texture(texture_id)
        {
            error!("Couldn't register external texture: {err}");
            return;
        }

        if let Err(err) = self
            .common
            .flutter
            .engine
            .mark_external_texture_frame_available(texture_id)
        {
            error!("Failed to mark external texture frame available: {err}");
            return;
        }

        self.common
            .textures_by_texture_id
            .insert(texture_id, texture.clone());

        self.common
            .current_texture_by_surface_id
            .insert(surface_id, (texture_id, texture.clone()));

        if is_new {
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
                    logical_size.w as i64,
                    logical_size.h as i64,
                ])))),
                None,
            );
        }
    }
}

fn ensure_initial_configure(
    surface: &WlSurface,
    initial_window_size: Option<Size<i32, Logical>>,
    space: &Space<Window>,
    popups: &mut PopupManager,
) {
    if let Some(window) = space
        .elements()
        .find(|window| window.wl_surface().map(|s| &*s == surface).unwrap_or(false))
        .cloned()
    {
        if let Some(toplevel) = window.toplevel()
            && !toplevel.is_initial_configure_sent()
        {
            toplevel.with_pending_state(|states| {
                states.size = initial_window_size;
            });
            toplevel.send_configure();
        }

        return;
    }

    if let Some(popup) = popups.find_popup(surface) {
        let popup = match popup {
            PopupKind::Xdg(ref popup) => popup,
            // Doesn't require configure
            PopupKind::InputMethod(ref _input_popup) => {
                return;
            }
        };

        if !popup.is_initial_configure_sent() {
            // NOTE: This should never fail as the initial configure is always
            // allowed.
            popup.send_configure().expect("initial configure failed");
        }

        return;
    };
}

fn render_window<R>(
    window: &Window,
    renderer: &mut R,
    scale: f64,
) -> Vec<WaylandSurfaceRenderElement<R>>
where
    R: Renderer + ImportAll,
    R::TextureId: Clone + 'static,
{
    match window.underlying_surface() {
        WindowSurface::Wayland(s) => render_elements_from_surface_tree(
            renderer,
            s.wl_surface(),
            (0, 0),
            (scale, scale),
            1.0,
            Kind::Unspecified,
        ),
        WindowSurface::X11(s) => {
            s.render_elements(renderer, (0, 0).into(), (scale, scale).into(), 1.0)
        }
    }
}

delegate_compositor!(State);
