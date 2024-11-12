use std::collections::HashMap;
use std::env::{remove_var, set_var};
use std::sync::Arc;
use std::time::Duration;
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::backend::input::{KeyState};
use smithay::backend::renderer::gles::ffi::{Gles2, RGBA8};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::input::pointer::{PointerHandle};
use smithay::input::{Seat, SeatState};
use smithay::input::keyboard::KeyboardHandle;
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::calloop::{channel, Interest, LoopHandle, LoopSignal, Mode, PostAction};
use smithay::reexports::calloop::channel::{Channel, Sender};
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

use platform_channels::encodable_value::EncodableValue;
use platform_channels::method_call::MethodCall;
use platform_channels::method_result::MethodResult;
use crate::flutter_engine::{Baton, FlutterEngine, KeyEvent};
use crate::mouse_button_tracker::FLUTTER_TO_LINUX_MOUSE_BUTTONS;
use crate::texture_swap_chain::TextureSwapChain;
use crate::{ClientState};
use crate::state::State;

pub struct Common {
    pub should_stop: bool,

    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, State>,
    pub loop_signal: LoopSignal,
    pub clock: Clock<Monotonic>,

    pub seat: Seat<State>,
    pub seat_state: SeatState<State>,
    pub data_device_state: DataDeviceState,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub dmabuf_state: DmabufState,
    pub pointer: PointerHandle<State>,
    pub keyboard: KeyboardHandle<State>,

    pub flutter_engine: Box<FlutterEngine>,
    pub tx_platform_message: Option<Sender<(MethodCall, Box<dyn MethodResult>)>>,
    pub tx_flutter_handled_key_events: Sender<(KeyEvent, bool)>,
    pub tx_fbo: Sender<Option<Dmabuf>>,
    pub baton: Option<Baton>,

    next_view_id: u64,
    next_texture_id: i64,

    pub mouse_position: (f64, f64),
    pub view_id_under_cursor: Option<u64>,
    pub is_next_vblank_scheduled: bool,

    pub imported_dmabufs: Vec<Dmabuf>,
    pub gles_renderer: GlesRenderer,
    pub gl: Gles2,
    pub surfaces: HashMap<u64, WlSurface>,
    pub xdg_toplevels: HashMap<u64, ToplevelSurface>,
    pub xdg_popups: HashMap<u64, PopupSurface>,
    pub texture_ids_per_view_id: HashMap<u64, Vec<i64>>,
    pub view_id_per_texture_id: HashMap<i64, u64>,
    pub texture_swapchains: HashMap<i64, TextureSwapChain>,
}

impl Common {
    pub fn new(
        display: Display<State>,
        loop_handle: LoopHandle<'static, State>,
        loop_signal: LoopSignal,
        seat_name: String,
        dmabuf_state: DmabufState,
        flutter_engine: Box<FlutterEngine>,
        tx_fbo: Sender<Option<Dmabuf>>,
        rx_baton: Channel<Baton>,
        rx_request_external_texture_name: Channel<i64>,
        tx_external_texture_name: Sender<(u32, u32)>,
        gles_renderer: GlesRenderer,
        gl: Gles2,
    ) -> Common {
        let display_handle = display.handle();
        let clock = Clock::new();
        let compositor_state = CompositorState::new::<State>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
        let shm_state = ShmState::new::<State>(&display_handle, vec![]);

        // init input
        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&display_handle, seat_name);

        let keyboard = seat.add_keyboard(Default::default(), 200, 25).unwrap();
        let pointer = seat.add_pointer();

        let data_device_state = DataDeviceState::new::<State>(&display_handle);

        let source = ListeningSocketSource::new_auto().unwrap();
        let socket_name = source.socket_name().to_string_lossy().into_owned();
        loop_handle
            .insert_source(source, |client_stream, _, data| {
                if let Err(err) = data
                    .common.display_handle
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
                |_, display, mut data| {
                    profiling::scope!("dispatch_clients");
                    // Safety: we don't drop the display
                    unsafe {
                        display.get_mut().dispatch_clients(&mut data).unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .expect("Failed to init wayland server source");

        loop_handle.insert_source(rx_baton, move |baton, _, data| {
            if let Msg(baton) = baton {
                data.common.baton = Some(baton);
            }
            if data.common.is_next_vblank_scheduled {
                return;
            }
            if let Some(baton) = data.common.baton.take() {
                data.common.flutter_engine.on_vsync(baton).unwrap();
            }
        }).unwrap();

        loop_handle.insert_source(rx_request_external_texture_name, move |event, _, data| {
            if let Msg(texture_id) = event {
                let texture_swap_chain = data.common.texture_swapchains.get_mut(&texture_id);
                let texture_id = match texture_swap_chain {
                    Some(texture) => {
                        let texture = texture.start_read();
                        texture.tex_id()
                    }
                    None => 0,
                };
                let _ = tx_external_texture_name.send((texture_id, RGBA8));
            }
        }).unwrap();

        let (tx_platform_message, rx_platform_message) = channel::channel::<(MethodCall, Box<dyn MethodResult>)>();

        Self::register_platform_message_handler(&loop_handle, rx_platform_message);

        let (tx_flutter_handled_key_events, rx_flutter_handled_key_events) = channel::channel::<(KeyEvent, bool)>();

        Self::register_flutter_handled_key_events_handler(&loop_handle, rx_flutter_handled_key_events);

        Self {
            should_stop: false,
            tx_fbo,
            baton: None,
            loop_signal,
            display_handle,
            loop_handle,
            clock,
            mouse_position: (0.0, 0.0),
            view_id_under_cursor: None,
            is_next_vblank_scheduled: false,
            compositor_state,
            xdg_shell_state,
            shm_state,
            flutter_engine,
            dmabuf_state,
            seat,
            seat_state,
            data_device_state,
            pointer,
            keyboard,
            next_view_id: 1,
            next_texture_id: 1,
            imported_dmabufs: Vec::new(),
            gles_renderer,
            gl,
            surfaces: HashMap::new(),
            xdg_toplevels: HashMap::new(),
            xdg_popups: HashMap::new(),
            texture_ids_per_view_id: HashMap::new(),
            view_id_per_texture_id: HashMap::new(),
            texture_swapchains: HashMap::new(),
            tx_platform_message: Some(tx_platform_message),
            tx_flutter_handled_key_events,
        }
    }

    pub(crate) fn now_ms(&self) -> u32 {
        Duration::from(self.clock.now()).as_millis() as u32
    }

    fn register_platform_message_handler(
        loop_handle: &LoopHandle<'static, State>,
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

                            data.pointer_hover(view_id, x, y);

                            result.success(None);
                        }
                        "pointer_exit" => {
                            data.pointer_exit();

                            result.success(None);
                        }
                        "mouse_button_event" => {
                            let args = method_call.arguments().unwrap();
                            let button = get_value(args, "button").long_value().unwrap() as u32;
                            let is_pressed = *extract!(get_value(args, "is_pressed"), EncodableValue::Bool);

                            data.mouse_button_event(
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

                            data.activate_window(view_id, activate);

                            result.success(None);
                        }
                        "resize_window" => {
                            let args = method_call.arguments().unwrap();
                            let view_id = get_value(args, "view_id").long_value().unwrap() as u64;
                            let width = get_value(args, "width").long_value().unwrap() as i32;
                            let height = get_value(args, "height").long_value().unwrap() as i32;

                            data.resize_window(view_id, width, height);

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

    fn register_flutter_handled_key_events_handler(
        loop_handle: &LoopHandle<State>,
        rx_flutter_handled_key_events: Channel<(KeyEvent, bool)>,
    ) {
        loop_handle
            .insert_source(
                rx_flutter_handled_key_events,
                |event, (), mut data| {
                    let Msg((key_event, handled)) = event else {
                        return;
                    };

                    if handled {
                        // Flutter consumed this event. Probably a keyboard shortcut.
                        return;
                    }

                    let text_input = &mut data.common.flutter_engine.text_input;

                    if text_input.is_active() {
                        if key_event.state == KeyState::Pressed
                            && !key_event.mods.ctrl
                            && !key_event.mods.alt
                        {
                            // text_input.press_key(key_event.key_code.raw(), key_event.codepoint);
                        }
                        // It doesn't matter if the text field captured the key event or not.
                        // As long as it stays active, don't forward events to the Wayland client.
                        return;
                    }

                    // The compositor was not interested in this event,
                    // so we forward it to the Wayland client in focus
                    // if there is one.
                    let keyboard = data.common.keyboard.clone();
                    keyboard.input_forward(
                        &mut data,
                        key_event.key_code,
                        key_event.state,
                        SERIAL_COUNTER.next_serial(),
                        key_event.time,
                        key_event.mods_changed,
                    );
                },
            )
            .expect("Failed to init wayland server source");
    }
}

impl Common {
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

pub struct MySurfaceState {
    pub view_id: u64,
    pub old_texture_size: Option<Size<i32, BufferCoords>>,
}
