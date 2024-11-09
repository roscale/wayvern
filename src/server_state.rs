use std::collections::HashMap;
use std::env::{remove_var, set_var};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::input::ButtonState;
use smithay::backend::renderer::gles::ffi::Gles2;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::input::pointer::{ButtonEvent, MotionEvent, PointerHandle};
use smithay::input::{Seat, SeatState};
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{channel, Interest, LoopHandle, Mode, PostAction};
use smithay::reexports::calloop::channel::Channel;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::Display;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::utils::{Buffer as BufferCoords, Clock, Monotonic, Size, SERIAL_COUNTER};
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::dmabuf::DmabufState;
use smithay::wayland::selection::data_device::DataDeviceState;
use smithay::wayland::shell::xdg::PopupSurface;
use smithay::wayland::shell::xdg::ToplevelSurface;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shm::ShmState;
use smithay::wayland::socket::ListeningSocketSource;
use tracing::{info, warn};

use crate::backends::Backend;
use platform_channels::encodable_value::EncodableValue;
use platform_channels::method_call::MethodCall;
use platform_channels::method_result::MethodResult;
use crate::flutter_engine::FlutterEngine;
use crate::mouse_button_tracker::FLUTTER_TO_LINUX_MOUSE_BUTTONS;
use crate::texture_swap_chain::TextureSwapChain;
use crate::{CalloopData, ClientState};

pub struct ServerState<BackendData: Backend + 'static> {
    pub running: Arc<AtomicBool>,
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
    pub clock: Clock<Monotonic>,

    pub seat: Seat<ServerState<BackendData>>,
    pub seat_state: SeatState<ServerState<BackendData>>,
    pub data_device_state: DataDeviceState,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub dmabuf_state: Option<DmabufState>,
    pub pointer: PointerHandle<ServerState<BackendData>>,

    pub backend_data: Box<BackendData>,

    pub flutter_engine: Option<Box<FlutterEngine>>,
    pub tx_platform_message: Option<channel::Sender<(MethodCall, Box<dyn MethodResult>)>>,
    next_view_id: u64,
    next_texture_id: i64,

    pub mouse_position: (f64, f64),
    pub view_id_under_cursor: Option<u64>,
    pub is_next_vblank_scheduled: bool,

    pub imported_dmabufs: Vec<Dmabuf>,
    pub gles_renderer: Option<GlesRenderer>,
    pub gl: Option<Gles2>,
    pub surfaces: HashMap<u64, WlSurface>,
    pub xdg_toplevels: HashMap<u64, ToplevelSurface>,
    pub xdg_popups: HashMap<u64, PopupSurface>,
    pub texture_ids_per_view_id: HashMap<u64, Vec<i64>>,
    pub view_id_per_texture_id: HashMap<i64, u64>,
    pub texture_swapchains: HashMap<i64, TextureSwapChain>,
}

impl<BackendData: Backend + 'static> ServerState<BackendData> {
    pub fn new(
        display: Display<ServerState<BackendData>>,
        loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
        backend_data: BackendData,
        dmabuf_state: Option<DmabufState>,
    ) -> ServerState<BackendData> {
        let display_handle = display.handle();
        let clock = Clock::new();
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);

        // init input
        let mut seat_state = SeatState::new();
        let seat_name = backend_data.seat_name();
        let mut seat = seat_state.new_wl_seat(&display_handle, seat_name.clone());
        seat.add_keyboard(Default::default(), 200, 200).unwrap();
        let pointer = seat.add_pointer();

        let data_device_state = DataDeviceState::new::<Self>(&display_handle);

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
        set_var("XDG_SESSION_TYPE", "wayland");
        set_var("GDK_BACKEND", "wayland"); // Force GTK apps to run on Wayland.
        set_var("QT_QPA_PLATFORM", "wayland"); // Force QT apps to run on Wayland.

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

        let (tx_platform_message, rx_platform_message) = channel::channel::<(MethodCall, Box<dyn MethodResult>)>();

        Self::register_platform_message_handler(&loop_handle, rx_platform_message);

        Self {
            running: Arc::new(AtomicBool::new(true)),
            display_handle,
            loop_handle,
            clock,
            backend_data: Box::new(backend_data),
            mouse_position: (0.0, 0.0),
            view_id_under_cursor: None,
            is_next_vblank_scheduled: false,
            compositor_state,
            xdg_shell_state,
            shm_state,
            flutter_engine: None,
            dmabuf_state,
            seat,
            seat_state,
            data_device_state,
            pointer,
            next_view_id: 1,
            next_texture_id: 1,
            imported_dmabufs: Vec::new(),
            gles_renderer: None,
            gl: None,
            surfaces: HashMap::new(),
            xdg_toplevels: HashMap::new(),
            xdg_popups: HashMap::new(),
            texture_ids_per_view_id: HashMap::new(),
            view_id_per_texture_id: HashMap::new(),
            texture_swapchains: HashMap::new(),
            tx_platform_message: Some(tx_platform_message),
        }
    }

    fn now_ms(&self) -> u32 {
        Duration::from(self.clock.now()).as_millis() as u32
    }

    fn register_platform_message_handler(
        loop_handle: &LoopHandle<'static, CalloopData<BackendData>>,
        rx_platform_message: Channel<(MethodCall, Box<dyn MethodResult>)>,
    ) {
        macro_rules! extract {
            ($e:expr, $p:path) => {
                match $e {
                    $p(value) => value,
                    _ => panic!("Failed to extract value"),
                }
            };
        }

        fn get_value<'a>(map: &'a EncodableValue, key: &str) -> &'a EncodableValue {
            let map = extract!(map, EncodableValue::Map);
            for (k, v) in map {
                if let EncodableValue::String(k) = k {
                    if k == key {
                        return v;
                    }
                }
            }
            panic!("Key {} not found in map", key);
        }

        loop_handle
            .insert_source(
                rx_platform_message,
                |event, (), data| {
                    let Msg((method_call, mut result)) = event else {
                        return;
                    };

                    match method_call.method() {
                        "pointer_hover" => {
                            let args = method_call.arguments().unwrap();
                            let view_id = get_value(args, "view_id").long_value().unwrap() as u64;
                            let x = *extract!(get_value(args, "x"), EncodableValue::Double);
                            let y = *extract!(get_value(args, "y"), EncodableValue::Double);

                            data.state.pointer_hover(view_id, x, y);

                            result.success(None);
                        }
                        "pointer_exit" => {
                            data.state.pointer_exit();

                            result.success(None);
                        }
                        "mouse_button_event" => {
                            let args = method_call.arguments().unwrap();
                            let button = get_value(args, "button").long_value().unwrap() as u32;
                            let is_pressed = *extract!(get_value(args, "is_pressed"), EncodableValue::Bool);

                            data.state.mouse_button_event(
                                *FLUTTER_TO_LINUX_MOUSE_BUTTONS.get(&(button)).unwrap() as u32,
                                is_pressed,
                            );

                            result.success(None);
                        }
                        "activate_window" => {
                            let args = method_call.arguments().unwrap();
                            let args = extract!(args, EncodableValue::List);
                            let view_id = args[0].long_value().unwrap() as u64;
                            let activate = extract!(args[1], EncodableValue::Bool);

                            data.state.activate_window(view_id, activate);

                            result.success(None);
                        }
                        "resize_window" => {
                            let args = method_call.arguments().unwrap();
                            let view_id = get_value(args, "view_id").long_value().unwrap() as u64;
                            let width = get_value(args, "width").long_value().unwrap() as i32;
                            let height = get_value(args, "height").long_value().unwrap() as i32;

                            data.state.resize_window(view_id, width, height);

                            result.success(None);
                        }
                        _ => {
                            result.success(None);
                        }
                    }
                },
            )
            .expect("Failed to init wayland server source");
    }

    fn pointer_hover(&mut self, view_id: u64, x: f64, y: f64) {
        let pointer = self.pointer.clone();

        self.view_id_under_cursor = Some(view_id);

        let Some(surface) = self.surfaces.get(&view_id).cloned() else {
            return;
        };

        pointer.motion(
            self,
            Some((surface.clone(), (0.0, 0.0).into())),
            &MotionEvent {
                location: (x, y).into(),
                serial: SERIAL_COUNTER.next_serial(),
                time: self.now_ms(),
            },
        );
        pointer.frame(self);
    }

    fn pointer_exit(&mut self) {
        let pointer = self.pointer.clone();

        self.view_id_under_cursor = None;

        pointer.motion(
            self,
            None,
            &MotionEvent {
                location: (0.0, 0.0).into(),
                serial: SERIAL_COUNTER.next_serial(),
                time: self.now_ms(),
            },
        );
        pointer.frame(self);
    }

    fn mouse_button_event(&mut self, button: u32, is_pressed: bool) {
        let pointer = self.pointer.clone();

        pointer.button(
            self,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: self.now_ms(),
                button,
                state: if is_pressed { ButtonState::Pressed } else { ButtonState::Released },
            },
        );
        pointer.frame(self);
    }

    fn activate_window(&mut self, view_id: u64, activate: bool) {
        let pointer = self.seat.get_pointer().unwrap();
        let keyboard = self.seat.get_keyboard().unwrap();

        let serial = SERIAL_COUNTER.next_serial();

        if pointer.is_grabbed() {
            return;
        }

        let Some(toplevel) = self.xdg_toplevels.get(&view_id).cloned() else {
            return;
        };

        toplevel.with_pending_state(|state| {
            if activate {
                state.states.set(xdg_toplevel::State::Activated);
            } else {
                state.states.unset(xdg_toplevel::State::Activated);
            }
        });
        keyboard.set_focus(self, Some(toplevel.wl_surface().clone()), serial);

        for toplevel in self.xdg_toplevels.values() {
            toplevel.send_pending_configure();
        }
    }

    fn resize_window(&mut self, view_id: u64, width: i32, height: i32) {
        let Some(surface) = self.xdg_toplevels.get(&view_id).cloned() else {
            return;
        };

        surface.with_pending_state(|state| {
            state.size = Some((width, height).into());
        });
        surface.send_pending_configure();
    }
}

impl<BackendData: Backend + 'static> ServerState<BackendData> {
    pub fn get_new_view_id(&mut self) -> u64 {
        let view_id = self.next_view_id;
        self.next_view_id += 1;
        view_id
    }

    pub fn get_new_texture_id(&mut self) -> i64 {
        let texture_id = self.next_texture_id;
        self.next_texture_id += 1;
        texture_id
    }
}

impl<BackendData: Backend + 'static> ServerState<BackendData> {
    pub fn flutter_engine(&self) -> &FlutterEngine {
        self.flutter_engine.as_ref().unwrap()
    }
    pub fn flutter_engine_mut(&mut self) -> &mut FlutterEngine {
        self.flutter_engine.as_mut().unwrap()
    }
}

pub struct MySurfaceState {
    pub view_id: u64,
    pub old_texture_size: Option<Size<i32, BufferCoords>>,
}
