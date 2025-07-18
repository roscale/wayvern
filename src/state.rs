use std::path::PathBuf;

use smithay::{
    backend::{allocator::dmabuf::Dmabuf, drm::DrmNode, renderer::ImportDma},
    reexports::{
        calloop::LoopSignal,
        wayland_server::{
            Client, DisplayHandle,
            backend::{ClientData, ClientId, DisconnectReason},
        },
    },
    wayland::dmabuf::{DmabufGlobal, DmabufState},
};

use crate::x11::X11State;

pub struct State {
    pub common: Common,
    pub backend: BackendState,
}

pub struct Common {
    pub display_handle: DisplayHandle,
    pub should_stop: bool,
    pub event_loop_signal: LoopSignal,
    pub dmabuf_state: DmabufState,
    pub flutter_project: PathBuf,
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
        Self {
            common: Common {
                display_handle,
                should_stop: false,
                event_loop_signal: loop_signal,
                dmabuf_state: DmabufState::new(),
                flutter_project,
            },
            backend: BackendState::None,
        }
    }

    pub(crate) fn new_client_state(&self) -> ClientState {
        ClientState {
            loop_signal: self.common.event_loop_signal.clone(),
        }
    }
}
