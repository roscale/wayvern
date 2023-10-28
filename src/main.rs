use std::{env, sync::{
    Arc,
    atomic::AtomicBool,
}};

use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::utils::on_commit_buffer_handler,
    },
    delegate_compositor,
    delegate_dmabuf,
    delegate_output,
    delegate_shm,
    delegate_xdg_shell,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            Client,
            DisplayHandle,
            protocol::{
                wl_buffer,
                wl_seat,
                wl_surface::{self, WlSurface},
            },
        },
    },
    utils::Serial,
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState, SurfaceAttributes,
            TraversalAction, with_surface_tree_downward,
        },
        dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportError},
        shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
    },
};
use smithay::reexports::calloop::{channel, Dispatcher, EventSource, LoopHandle};
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::wayland_server::Display;
use smithay::utils::{Clock, Monotonic};
use crate::flutter_engine::embedder::FlutterEngineRunTask;

use crate::flutter_engine::FlutterEngine;
use crate::flutter_engine::task_runner::TaskRunner;
use crate::mouse_button_tracker::MouseButtonTracker;

mod flutter_engine;
mod x11_client;
mod gles_framebuffer_importer;
mod mouse_button_tracker;
mod drm_backend;
mod input_handling;
mod cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    if env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok() {
        x11_client::run_x11_client();
    } else {
        drm_backend::run_drm_backend();
    }

    Ok(())
}

pub trait Backend {}

pub struct GlobalState<BackendData: Backend + 'static + ?Sized> {
    pub running: Arc<AtomicBool>,
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
    pub clock: Clock<Monotonic>,
    pub backend_data: Box<BackendData>,
    // space: Space<WindowElement>,

    pub flutter_engine: FlutterEngine,
    pub mouse_button_tracker: MouseButtonTracker,
    pub mouse_position: (f64, f64),
    pub is_next_vblank_scheduled: bool,

    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
}

impl<BackendData: Backend + 'static> GlobalState<BackendData> {
    pub fn new(
        display: Display<GlobalState<BackendData>>,
        loop_handle: LoopHandle<'static, CalloopData<BackendData>>,
        backend_data: BackendData,
    ) -> GlobalState<BackendData> {
        let display_handle = display.handle();
        let clock = Clock::new().expect("failed to initialize clock");
        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);

        let task_runner_timer_dispatcher = Dispatcher::new(Timer::immediate(), |deadline, _, data: &mut CalloopData<BackendData>| {
            let handle = data.state.flutter_engine.handle;
            let duration = data.state.flutter_engine.task_runner.execute_expired_tasks(&|task| {
                unsafe { FlutterEngineRunTask(handle, task as *const _) };
            });
            TimeoutAction::ToDuration(duration)
        });

        let task_runner_timer_registration_token = loop_handle.register_dispatcher(task_runner_timer_dispatcher.clone()).unwrap();
        let (reschedule_timer_tx, reschedule_timer_rx) = channel::channel();

        loop_handle.insert_source(reschedule_timer_rx, move |event, _, data: &mut CalloopData<BackendData>| {
            if let Msg(duration) = event {
                task_runner_timer_dispatcher.as_source_mut().set_duration(duration);
                data.state.loop_handle.update(&task_runner_timer_registration_token).unwrap();
            }
        }).unwrap();

        Self {
            running: Arc::new(AtomicBool::new(true)),
            display_handle,
            loop_handle,
            clock,
            backend_data: Box::new(backend_data),
            flutter_engine: FlutterEngine::new(TaskRunner::new(reschedule_timer_tx)),
            mouse_button_tracker: MouseButtonTracker::new(),
            mouse_position: (0.0, 0.0),
            is_next_vblank_scheduled: false,
            compositor_state,
            xdg_shell_state,
            shm_state,
        }
    }
}

impl<BackendData: Backend> BufferHandler for GlobalState<BackendData> {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl<BackendData: Backend> XdgShellHandler for GlobalState<BackendData> {
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

impl<BackendData: Backend> CompositorHandler for GlobalState<BackendData> {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl<BackendData: Backend> ShmHandler for GlobalState<BackendData> {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl<BackendData: Backend> DmabufHandler for GlobalState<BackendData> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        todo!()
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, _dmabuf: Dmabuf) -> Result<(), ImportError> {
        todo!()
    }
}

pub struct CalloopData<BackendData: Backend + 'static + ?Sized> {
    pub state: GlobalState<BackendData>,
    pub tx_fbo: channel::Sender<Option<Dmabuf>>,
    pub baton: Option<flutter_engine::Baton>,
}

pub fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been commited
            for callback in states
                .cached_state
                .current::<SurfaceAttributes>()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("disconnected");
    }
}

// Macros used to delegate protocol handling to types in the app state.
delegate_xdg_shell!(@<BackendData: Backend + 'static> GlobalState<BackendData>);
delegate_compositor!(@<BackendData: Backend + 'static> GlobalState<BackendData>);
delegate_shm!(@<BackendData: Backend + 'static> GlobalState<BackendData>);
delegate_dmabuf!(@<BackendData: Backend + 'static> GlobalState<BackendData>);
delegate_output!(@<BackendData: Backend + 'static> GlobalState<BackendData>);

// delegate_seat!(App);
// delegate_data_device!(App);
