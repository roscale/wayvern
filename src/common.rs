use std::collections::HashMap;
use std::env::{remove_var, set_var};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use smithay::backend::allocator::dmabuf::Dmabuf;
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
use smithay::utils::{Buffer as BufferCoords, Clock, Monotonic, Size};
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
use platform_channels::method_channel::MethodChannel;
use platform_channels::method_result::MethodResult;
use platform_channels::standard_method_codec::StandardMethodCodec;
use crate::flutter_engine::{Baton, FlutterEngine, KeyEvent};
use crate::texture_swap_chain::TextureSwapChain;
use crate::{ClientState};
use crate::input_handling::register_flutter_handled_key_event_handler;
use crate::platform_message_handler::register_platform_message_handler;
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
    pub tx_flutter_handled_key_event: Sender<(KeyEvent, bool)>,
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
        mut flutter_engine: Box<FlutterEngine>,
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

        let (tx_platform_message, rx_platform_message) = channel::channel::<(MethodCall, Box<dyn MethodResult>)>();

        let codec = Rc::new(StandardMethodCodec::new());
        let mut platform_method_channel = MethodChannel::<EncodableValue>::new(
            flutter_engine.binary_messenger.as_mut().unwrap(),
            "platform".to_string(),
            codec,
        );
        // TODO: Provide a way to specify a channel directly, without registering a callback.
        platform_method_channel.set_method_call_handler(Some(Box::new(move |method_call, result| {
            tx_platform_message.send((method_call, result)).unwrap();
        })));

        register_platform_message_handler(&loop_handle, rx_platform_message);

        let (tx_flutter_handled_key_event, rx_flutter_handled_key_event) = channel::channel::<(KeyEvent, bool)>();

        register_flutter_handled_key_event_handler(&loop_handle, rx_flutter_handled_key_event);

        loop_handle.insert_source(Generic::new(display, Interest::READ, Mode::Level), |_, display, data| {
            profiling::scope!("dispatch_clients");
            // Safety: we don't drop the display
            unsafe {
                display.get_mut().dispatch_clients(data).unwrap();
            }
            Ok(PostAction::Continue)
        }).unwrap();

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
            tx_flutter_handled_key_event,
        }
    }

    pub(crate) fn now_ms(&self) -> u32 {
        Duration::from(self.clock.now()).as_millis() as u32
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
