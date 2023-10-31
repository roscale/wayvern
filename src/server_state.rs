use std::collections::HashMap;
use std::env::{remove_var, set_var};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use smithay::{delegate_compositor, delegate_dmabuf, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell};
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::renderer::{ImportAll, Texture};
use smithay::input::{Seat, SeatHandler, SeatState};
use smithay::input::pointer::CursorImageStatus;
use smithay::reexports::calloop::{Interest, LoopHandle, Mode, PostAction};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::{Client, Display, DisplayHandle};
use smithay::reexports::wayland_server::protocol::{wl_buffer, wl_seat};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Clock, Monotonic, Rectangle, Serial, Size};
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::compositor::{BufferAssignment, CompositorClientState, CompositorHandler, CompositorState, SurfaceAttributes, with_states};
use smithay::wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportError};
use smithay::wayland::shell::xdg;
use smithay::wayland::shell::xdg::{PopupSurface, PositionerState, SurfaceCachedState, ToplevelSurface, XdgShellHandler, XdgShellState};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::wayland::socket::ListeningSocketSource;
use tracing::{info, warn};

use crate::{Backend, CalloopData, ClientState};
use crate::flutter_engine::FlutterEngine;
use crate::flutter_engine::platform_channels::method_channel::MethodChannel;
use crate::flutter_engine::platform_channels::standard_method_codec::StandardMethodCodec;
use crate::flutter_engine::wayland_messages::{SurfaceCommitMessage, XdgSurfaceCommitMessage};

pub struct ServerState<BackendData: Backend + 'static> {
    pub running: Arc<AtomicBool>,
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
    pub clock: Clock<Monotonic>,
    pub seat: Seat<ServerState<BackendData>>,
    pub seat_state: SeatState<ServerState<BackendData>>,
    pub backend_data: Box<BackendData>,
    pub flutter_engine: Option<Box<FlutterEngine>>,
    pub next_view_id: u64,
    pub next_texture_id: u64,
    // space: Space<WindowElement>,

    pub mouse_position: (f64, f64),
    pub is_next_vblank_scheduled: bool,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub dmabuf_state: Option<DmabufState>,

    pub gles_texture_per_texture_id: HashMap<i64, GlesTexture>,
    pub imported_dmabufs: Vec<Dmabuf>,
    pub gles_renderer: Option<GlesRenderer>,
}

impl<BackendData: Backend + 'static> ServerState<BackendData> {
    pub fn flutter_engine(&self) -> &FlutterEngine {
        self.flutter_engine.as_ref().unwrap()
    }
    pub fn flutter_engine_mut(&mut self) -> &mut FlutterEngine {
        self.flutter_engine.as_mut().unwrap()
    }
}

// Macros used to delegate protocol handling to types in the app state.
delegate_compositor!(@<BackendData: Backend + 'static> ServerState<BackendData>);
delegate_xdg_shell!(@<BackendData: Backend + 'static> ServerState<BackendData>);
delegate_shm!(@<BackendData: Backend + 'static> ServerState<BackendData>);
delegate_dmabuf!(@<BackendData: Backend + 'static> ServerState<BackendData>);
delegate_output!(@<BackendData: Backend + 'static> ServerState<BackendData>);
delegate_seat!(@<BackendData: Backend + 'static> ServerState<BackendData>);
// delegate_data_device!(App);

impl<BackendData: Backend + 'static> ServerState<BackendData> {
    pub fn new(
        display: Display<ServerState<BackendData>>,
        loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
        backend_data: BackendData,
        dmabuf_state: Option<DmabufState>,
    ) -> ServerState<BackendData> {
        let display_handle = display.handle();
        let clock = Clock::new().expect("failed to initialize clock");
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);

        // init input
        let mut seat_state = SeatState::new();
        let seat_name = backend_data.seat_name();
        let mut seat = seat_state.new_wl_seat(&display_handle, seat_name.clone());
        seat.add_keyboard(Default::default(), 200, 200).unwrap();

        // init wayland clients
        let source = ListeningSocketSource::new_auto().unwrap();
        let socket_name = source.socket_name().to_string_lossy().into_owned();
        loop_handle
            .insert_source(source, |client_stream, _, data| {
                if let Err(err) = data
                    .state.display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                {
                    warn!("Error adding wayland client: {}", err);
                };
            })
            .expect("Failed to init wayland socket source");

        info!(name = socket_name, "Listening on wayland socket");

        remove_var("DISPLAY");
        set_var("WAYLAND_DISPLAY", &socket_name);

        loop_handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, data| {
                    profiling::scope!("dispatch_clients");
                    // Safety: we don't drop the display
                    unsafe {
                        display.get_mut().dispatch_clients(&mut data.state).unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("Failed to init wayland server source");

        Self {
            running: Arc::new(AtomicBool::new(true)),
            display_handle,
            loop_handle,
            clock,
            backend_data: Box::new(backend_data),
            mouse_position: (0.0, 0.0),
            is_next_vblank_scheduled: false,
            compositor_state,
            xdg_shell_state,
            shm_state,
            flutter_engine: None,
            dmabuf_state,
            seat,
            seat_state,
            next_view_id: 1,
            next_texture_id: 1,
            gles_texture_per_texture_id: HashMap::new(),
            imported_dmabufs: Vec::new(),
            gles_renderer: None,
        }
    }
}

impl<BackendData: Backend> BufferHandler for ServerState<BackendData> {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl<BackendData: Backend> XdgShellHandler for ServerState<BackendData> {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        // Handle popup creation here
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Handle popup grab here
    }
}

pub struct MySurfaceState {
    pub view_id: u64,
}

impl<BackendData: Backend> CompositorHandler for ServerState<BackendData> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        // add_pre_commit_hook::<Self, _>(surface, move |state, _dh, surface| {
        //     let maybe_dmabuf = with_states(surface, |surface_data| {
        //         surface_data
        //             .cached_state
        //             .pending::<SurfaceAttributes>()
        //             .buffer
        //             .as_ref()
        //             .and_then(|assignment| match assignment {
        //                 BufferAssignment::NewBuffer(buffer) => get_dmabuf(buffer).ok(),
        //                 _ => None,
        //             })
        //     });
        //     if let Some(dmabuf) = maybe_dmabuf {
        //         dbg!("da");
        //     }
        // })
    }

    fn commit(&mut self, surface: &WlSurface) {
        // on_commit_buffer_handler::<Self>(surface);

        let commit_message = with_states(surface, |surface_data| {
            let role = surface_data.role;
            let state = surface_data.cached_state.current::<SurfaceAttributes>();

            let my_state = surface_data.data_map.get_or_insert(|| {
                let view_id = self.next_view_id;
                self.next_view_id += 1;
                MySurfaceState {
                    view_id,
                }
            });


            let texture = state.buffer
                .as_ref()
                .and_then(|assignment| match assignment {
                    BufferAssignment::NewBuffer(buffer) => {
                        self.gles_renderer.as_mut().unwrap().import_buffer(buffer, None, &[]).map(|t| t.ok()).flatten()
                    },
                    _ => None,
                });

            let (texture_id, size) = if let Some(texture) = texture {
                let size = texture.size();
                let texture_id = self.next_view_id;
                self.next_texture_id += 1;
                self.gles_texture_per_texture_id.insert(texture_id as i64, texture);
                let _ = self.flutter_engine_mut().register_external_texture(texture_id);
                let _ = self.flutter_engine_mut().mark_external_texture_frame_available(texture_id);
                (texture_id, Some(size))
            } else {
                (0, None)
            };

            SurfaceCommitMessage {
                view_id: my_state.view_id,
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
                            .pending::<SurfaceCachedState>()
                            .geometry;

                        Some(XdgSurfaceCommitMessage {
                            mapped: texture_id != 0,
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
                    },
                    _ => None,
                },
            }
        });

        let commit_message = commit_message.serialize();

        let codec = Rc::new(StandardMethodCodec::new());
        let mut method_channel = MethodChannel::new(
            self.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        dbg!(&commit_message);
        method_channel.invoke_method("commit_surface", Some(Box::new(commit_message)), None);
    }
}

impl<BackendData: Backend> ShmHandler for ServerState<BackendData> {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl<BackendData: Backend> DmabufHandler for ServerState<BackendData> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        self.dmabuf_state.as_mut().unwrap()
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, _dmabuf: Dmabuf) -> Result<(), ImportError> {
        self.imported_dmabufs.push(_dmabuf);
        Ok(())
    }
}

// impl DmabufHandler for ServerState<X11Data> {
//     fn dmabuf_state(&mut self) -> &mut DmabufState {
//         &mut self.dmabuf_state.as_mut().unwrap()
//     }
//
//     fn dmabuf_imported(&mut self, _global: &DmabufGlobal, dmabuf: Dmabuf) -> Result<(), ImportError> {
//         self.backend_data
//             .gles_renderer
//             .import_dmabuf(&dmabuf, None)
//             .map(|_| ())
//             .map_err(|_| ImportError::Failed)
//     }
// }

impl<BackendData: Backend> SeatHandler for ServerState<BackendData> {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<ServerState<BackendData>> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, seat: &Seat<Self>, target: Option<&WlSurface>) {

    }
    fn cursor_image(&mut self, _seat: &Seat<Self>, image: CursorImageStatus) {

    }
}
