use std::cell::RefCell;
use std::rc::Rc;
use smithay::backend::renderer::{ImportAll, Texture};
use smithay::delegate_compositor;
use smithay::reexports::wayland_server::Client;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Rectangle;
use smithay::wayland::compositor;
use smithay::wayland::compositor::{with_states, with_surface_tree_upward, BufferAssignment, CompositorClientState, CompositorHandler, CompositorState, SubsurfaceCachedState, SurfaceAttributes, TraversalAction};
use smithay::wayland::shell::xdg;
use smithay::wayland::shell::xdg::{SurfaceCachedState, XdgPopupSurfaceData, XdgToplevelSurfaceData};
use crate::{ClientState};
use crate::backends::Backend;
use crate::flutter_engine::platform_channels::encodable_value::EncodableValue;
use crate::flutter_engine::platform_channels::method_channel::MethodChannel;
use crate::flutter_engine::platform_channels::standard_method_codec::StandardMethodCodec;
use crate::flutter_engine::wayland_messages::{SubsurfaceCommitMessage, SurfaceCommitMessage, XdgPopupCommitMessage, XdgSurfaceCommitMessage};
use crate::server_state::{MySurfaceState, ServerState};

delegate_compositor!(@<BackendData: Backend + 'static> ServerState<BackendData>);

impl<BackendData: Backend> CompositorHandler for ServerState<BackendData> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        let view_id = self.get_new_view_id();
        with_states(surface, |surface_data| {
            surface_data.data_map.insert_if_missing(|| RefCell::new(MySurfaceState {
                view_id,
                old_texture_size: None,
            }));
        });
        self.surfaces.insert(view_id, surface.clone());

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("new_surface", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
        ]))), None);
    }

    fn new_subsurface(&mut self, surface: &WlSurface, parent: &WlSurface) {
        let view_id = with_states(surface, |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });

        let parent_view_id = with_states(parent, |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("new_subsurface", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
            (EncodableValue::String("parent".to_string()), EncodableValue::Int64(parent_view_id as i64)),
        ]))), None);
    }

    fn commit(&mut self, surface: &WlSurface) {
        self.send_initial_configure(surface);

        let mut commit_message = with_states(surface, |surface_data| {
            let role = surface_data.role;

            let mut state = surface_data.cached_state.get::<SurfaceAttributes>();
            let state = state.current();
            let my_state = surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap();

            let (view_id, old_texture_size) = {
                let my_state = my_state.borrow();
                (my_state.view_id, my_state.old_texture_size)
            };

            let texture = state.buffer
                .as_ref()
                .and_then(|assignment| match assignment {
                    BufferAssignment::NewBuffer(buffer) => {
                        self.gles_renderer.as_mut().unwrap().import_buffer(buffer, Some(surface_data), &[]).and_then(|t| t.ok())
                    }
                    _ => None,
                });

            let (texture_id, size) = if let Some(texture) = texture {
                unsafe { self.gl.as_ref().unwrap().Finish(); }

                let size = texture.size();

                let size_changed = match old_texture_size {
                    Some(old_size) => old_size != size,
                    None => true,
                };

                my_state.borrow_mut().old_texture_size = Some(size);

                let texture_id = match size_changed {
                    true => None,
                    false => self.texture_ids_per_view_id.get(&view_id).and_then(|v| v.last()).cloned(),
                };

                let texture_id = texture_id.unwrap_or_else(|| {
                    let texture_id = self.get_new_texture_id();
                    while self.texture_ids_per_view_id.entry(view_id).or_default().len() >= 2 {
                        self.texture_ids_per_view_id.entry(view_id).or_default().remove(0);
                    }

                    self.texture_ids_per_view_id.entry(view_id).or_default().push(texture_id);
                    self.view_id_per_texture_id.insert(texture_id, view_id);
                    self.flutter_engine_mut().register_external_texture(texture_id).unwrap();
                    texture_id
                });

                let swapchain = self.texture_swapchains.entry(texture_id).or_default();
                swapchain.commit(texture.clone());

                self.flutter_engine_mut().mark_external_texture_frame_available(texture_id).unwrap();

                (texture_id, Some(size))
            } else {
                (-1, None)
            };

            SurfaceCommitMessage {
                view_id,
                role,
                texture_id,
                buffer_delta: state.buffer_delta,
                buffer_size: size,
                scale: state.buffer_scale,
                input_region: state.input_region.clone(),
                xdg_surface: match role {
                    Some(xdg::XDG_TOPLEVEL_ROLE | xdg::XDG_POPUP_ROLE) => {
                        let geometry = surface_data
                            .cached_state
                            .get::<SurfaceCachedState>()
                            .current()
                            .geometry;

                        Some(XdgSurfaceCommitMessage {
                            role,
                            geometry: match geometry {
                                Some(geometry) => Some(geometry),
                                None => Some(Rectangle {
                                    loc: (0, 0).into(),
                                    size: match size {
                                        Some(size) => (size.w, size.h).into(),
                                        None => (0, 0).into(),
                                    },
                                }),
                            },
                        })
                    }
                    _ => None,
                },
                xdg_popup: match role {
                    Some(xdg::XDG_POPUP_ROLE) => {
                        let popup_data = surface_data.data_map.get::<XdgPopupSurfaceData>().unwrap().lock().unwrap();
                        let parent_id = popup_data.parent.as_ref().map(|surface| {
                            with_states(surface, |surface_data| {
                                surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
                            })
                        }).unwrap_or(0);

                        Some(XdgPopupCommitMessage {
                            parent_id,
                            geometry: popup_data.current.geometry,
                        })
                    }
                    _ => None,
                },
                subsurface: match role {
                    Some(compositor::SUBSURFACE_ROLE) => {
                        Some(SubsurfaceCommitMessage {
                            location: surface_data.cached_state.get::<SubsurfaceCachedState>().current().location,
                        })
                    }
                    _ => None,
                },
                subsurfaces_below: vec![],
                subsurfaces_above: vec![],
            }
        });

        let mut subsurfaces_below = vec![];
        let mut subsurfaces_above = vec![];
        let mut above = false;

        with_surface_tree_upward(surface, (), |child_surface, _, ()| {
            // Only traverse the direct children of the surface
            if child_surface == surface {
                TraversalAction::DoChildren(())
            } else {
                TraversalAction::SkipChildren
            }
        }, |child_surface, surface_data, ()| {
            if child_surface == surface {
                above = true;
                return;
            }

            let view_id = surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id;
            if above {
                subsurfaces_above.push(view_id);
            } else {
                subsurfaces_below.push(view_id);
            }
        }, |_, _, _| true);

        commit_message.subsurfaces_below = subsurfaces_below;
        commit_message.subsurfaces_above = subsurfaces_above;

        let commit_message = commit_message.serialize();

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("commit_surface", Some(Box::new(commit_message)), None);
    }

    fn destroyed(&mut self, _surface: &WlSurface) {
        let view_id = with_states(_surface, |surface_data| {
            surface_data.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });
        self.surfaces.remove(&view_id);

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        method_channel.invoke_method("destroy_surface", Some(Box::new(EncodableValue::Map(vec![
            (EncodableValue::String("view_id".to_string()), EncodableValue::Int64(view_id as i64)),
        ]))), None);
    }
}

impl<BackendData: Backend> ServerState<BackendData> {
    fn send_initial_configure(&self, surface: &WlSurface) {
        let view_id = with_states(surface, |states| {
            states.data_map.get::<RefCell<MySurfaceState>>().unwrap().borrow().view_id
        });

        if let Some(toplevel) = self.xdg_toplevels.get(&view_id) {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgToplevelSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                toplevel.send_configure();
            }
        }

        if let Some(popup) = self.xdg_popups.get(&view_id) {
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgPopupSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });

            if !initial_configure_sent {
                // NOTE: This should never fail as the initial configure is always
                // allowed.
                popup.send_configure().expect("initial configure failed");
            }
        }
    }
}