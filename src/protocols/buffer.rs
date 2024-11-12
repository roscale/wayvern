use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::reexports::wayland_server::protocol::wl_buffer;
use smithay::wayland::buffer::BufferHandler;
use smithay::wayland::dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier};
use smithay::wayland::shm::{ShmHandler, ShmState};
use smithay::{delegate_dmabuf, delegate_shm};
use crate::state::State;

delegate_shm!(State);
delegate_dmabuf!(State);

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.common.shm_state
    }
}

impl DmabufHandler for State {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.common.dmabuf_state
    }

    fn dmabuf_imported(&mut self, _global: &DmabufGlobal, _dmabuf: Dmabuf, notifier: ImportNotifier) {
        self.common.imported_dmabufs.push(_dmabuf);
        notifier.successful::<State>().expect("Failed to notify successful import");
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

