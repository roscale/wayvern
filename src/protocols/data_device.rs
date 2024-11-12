use smithay::delegate_data_device;
use smithay::wayland::selection::data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler};
use smithay::wayland::selection::SelectionHandler;
use crate::state::State;

delegate_data_device!(State);

impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.common.data_device_state
    }
}

impl SelectionHandler for State {
    type SelectionUserData = ();
}

impl ClientDndGrabHandler for State {}

impl ServerDndGrabHandler for State {}

