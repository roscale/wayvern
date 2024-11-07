use smithay::delegate_data_device;
use smithay::wayland::selection::data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler};
use smithay::wayland::selection::SelectionHandler;
use crate::backends::Backend;
use crate::server_state::ServerState;

delegate_data_device!(@<BackendData: Backend + 'static> ServerState<BackendData>);

impl<BackendData: Backend> DataDeviceHandler for ServerState<BackendData> {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl<BackendData: Backend> SelectionHandler for ServerState<BackendData> {
    type SelectionUserData = ();
}

impl<BackendData: Backend> ClientDndGrabHandler for ServerState<BackendData> {}

impl<BackendData: Backend> ServerDndGrabHandler for ServerState<BackendData> {}

