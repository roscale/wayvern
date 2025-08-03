use std::path::PathBuf;

use smithay::{
    backend::{allocator::dmabuf::Dmabuf, drm::DrmNode, renderer::ImportDma},
    desktop::Window,
    input::{Seat, SeatState},
    reexports::{
        calloop::LoopSignal,
        wayland_server::{
            Client, DisplayHandle,
            backend::{ClientData, ClientId, DisconnectReason},
        },
    },
    utils::{Logical, Size},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        dmabuf::{DmabufGlobal, DmabufState},
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

use crate::{
    flutter_engine_rs::Engine,
    utils::AutoDerefOption,
    x11::{UserData, X11State},
};

pub struct State {
    pub common: Common,
    pub backend: BackendState,
}

pub struct Common {
    pub display_handle: DisplayHandle,
    pub should_stop: bool,
    pub event_loop_signal: LoopSignal,

    pub flutter_project: PathBuf,
    pub flutter: AutoDerefOption<Flutter>,

    pub shm_state: ShmState,
    pub seat_state: SeatState<State>,
    pub seat: Seat<State>,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub dmabuf_state: DmabufState,

    pub next_window_id: usize,
    pub windows: Vec<Window>,
    pub initial_window_size: Option<Size<i32, Logical>>,
}

pub struct Flutter {
    pub engine: Engine<UserData>,
}

pub enum BackendState {
    X11(Box<X11State>),
    None,
}

impl BackendState {
    pub fn x11(&mut self) -> &mut X11State {
        match self {
            BackendState::X11(x11_state) => x11_state,
            _ => unreachable!("Called x11 in non x11 backend"),
        }
    }

    pub fn dmabuf_imported(
        &mut self,
        _client: Option<Client>,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
    ) -> Result<Option<DrmNode>, anyhow::Error> {
        match self {
            BackendState::X11(state) => {
                state.renderer.import_dmabuf(&dmabuf, None)?;
            }
            _ => unreachable!("No backend set when importing dmabuf"),
        };
        Ok(None)
    }
}

pub struct ClientState {
    pub compositor_client_state: CompositorClientState,
    pub loop_signal: LoopSignal,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        self.loop_signal.wakeup();
    }
}

impl State {
    pub(crate) fn new(
        display_handle: DisplayHandle,
        loop_signal: LoopSignal,
        flutter_project: PathBuf,
    ) -> Self {
        let mut seat_state = SeatState::new();

        // TODO(roscale): Different seat name for X11 and DRM modes.
        let mut seat = seat_state.new_wl_seat(&display_handle, "<TODO seat name>");
        seat.add_keyboard(Default::default(), 300, 25).unwrap();

        let shm_state = ShmState::new::<State>(&display_handle, vec![]);
        let compositor_state = CompositorState::new::<State>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
        let dmabuf_state = DmabufState::new();

        Self {
            common: Common {
                display_handle,
                should_stop: false,
                event_loop_signal: loop_signal,
                flutter_project,
                flutter: None.into(),
                shm_state,
                seat_state,
                seat,
                compositor_state,
                xdg_shell_state,
                dmabuf_state,
                next_window_id: 0,
                windows: Vec::new(),
                initial_window_size: None,
            },
            backend: BackendState::None,
        }
    }

    pub(crate) fn new_client_state(&self) -> ClientState {
        ClientState {
            compositor_client_state: Default::default(),
            loop_signal: self.common.event_loop_signal.clone(),
        }
    }
}
