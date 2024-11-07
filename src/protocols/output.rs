use smithay::delegate_output;
use smithay::wayland::output::OutputHandler;
use crate::backends::Backend;
use crate::server_state::ServerState;

delegate_output!(@<BackendData: Backend + 'static> ServerState<BackendData>);

impl<BackendData: Backend> OutputHandler for ServerState<BackendData> {}