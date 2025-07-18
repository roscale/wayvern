use smithay::{
    reexports::wayland_server::protocol::wl_buffer::WlBuffer, wayland::buffer::BufferHandler,
};

use crate::state::State;

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {
        // if let BackendData::Kms(kms_state) = &mut self.backend {
        //     for device in kms_state.drm_devices.values_mut() {
        //         if device.active_buffers.remove(&buffer.downgrade()) {
        //             if !device.in_use(kms_state.primary_node.read().unwrap().as_ref()) {
        //                 if let Err(err) = kms_state.refresh_used_devices() {
        //                     warn!(?err, "Failed to init devices.");
        //                 };
        //                 break;
        //             }
        //         }
        //     }
        // }
    }
}
