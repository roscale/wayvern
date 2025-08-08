use std::{collections::HashMap, path::PathBuf};

use smithay::{
    backend::renderer::gles::GlesTexture,
    desktop::{PopupManager, Space, Window, WindowSurface},
    input::{Seat, SeatState},
    output::{Mode, Output, PhysicalProperties, Scale, Subpixel},
    reexports::{
        calloop::LoopSignal,
        wayland_server::{
            DisplayHandle,
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
        },
    },
    utils::{Clock, Logical, Monotonic, Size, Transform},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        dmabuf::DmabufState,
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

    pub space: Space<Window>,
    pub popups: PopupManager,

    pub flutter_project: PathBuf,
    pub flutter: AutoDerefOption<Flutter>,

    pub clock: Clock<Monotonic>,

    pub output: Output,
    pub shm_state: ShmState,
    pub seat_state: SeatState<State>,
    pub seat: Seat<State>,
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub dmabuf_state: DmabufState,

    pub next_window_id: usize,
    pub next_texture_id: i64,

    pub surfaces_by_texture_id: HashMap<i64, WlSurface>,
    pub textures_by_texture_id: HashMap<i64, GlesTexture>,
    pub current_texture_by_surface_id: HashMap<usize, (i64, GlesTexture)>,
    pub textures: Vec<GlesTexture>,
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

    // pub fn dmabuf_imported(
    //     &mut self,
    //     _client: Option<Client>,
    //     _global: &DmabufGlobal,
    //     dmabuf: Dmabuf,
    // ) -> Result<Option<DrmNode>, anyhow::Error> {
    //     match self {
    //         BackendState::X11(state) => {
    //             state.renderer.import_dmabuf(&dmabuf, None)?;
    //         }
    //         _ => unreachable!("No backend set when importing dmabuf"),
    //     };
    //     Ok(None)
    // }
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

        let output = Output::new(
            "TODO output name".to_string(),
            PhysicalProperties {
                size: (1600, 1200).into(),
                subpixel: Subpixel::Unknown,
                make: "TODO make".to_string(),
                model: "TODO model".to_string(),
            },
        );

        let mode = Mode {
            size: (1600, 1200).into(),
            refresh: 144,
        };
        output.set_preferred(mode);
        output.change_current_state(Some(mode), None, None, None);

        output.create_global::<State>(&display_handle);

        let shm_state = ShmState::new::<State>(&display_handle, vec![]);
        let compositor_state = CompositorState::new::<State>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
        let dmabuf_state = DmabufState::new();

        Self {
            common: Common {
                display_handle,
                should_stop: false,
                event_loop_signal: loop_signal,
                space: Space::default(),
                popups: PopupManager::default(),
                flutter_project,
                flutter: None.into(),
                output,
                clock: Clock::new(),
                shm_state,
                seat_state,
                seat,
                compositor_state,
                xdg_shell_state,
                dmabuf_state,
                next_window_id: 1,
                next_texture_id: 1,
                surfaces_by_texture_id: HashMap::new(),
                textures_by_texture_id: HashMap::new(),
                current_texture_by_surface_id: HashMap::new(),
                textures: Vec::new(),
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
