use smithay::delegate_output;
use smithay::wayland::output::OutputHandler;
use crate::state::State;

delegate_output!(State);

impl OutputHandler for State {}
