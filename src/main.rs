use std::env;
use std::env::set_var;
use std::sync::Arc;
use log::{info, warn};
use crate::backends::drm::run_drm_backend;
use crate::backends::x11::run_x11_client;
use smithay::reexports::calloop::{EventLoop, Interest, LoopHandle, Mode, PostAction};
use smithay::reexports::calloop::generic::Generic;
use smithay::reexports::wayland_server::{Display, DisplayHandle};
use smithay::wayland::socket::ListeningSocketSource;
use crate::protocols::compositor::ClientState;
use crate::state::State;

mod flutter_engine;
mod gles_framebuffer_importer;
mod mouse_button_tracker;
mod input_handling;
mod cursor;
mod common;
mod texture_swap_chain;
mod protocols;
mod backends;
mod keyboard;
mod state;
mod platform_message_handler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let running_under_shell = env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();

    let event_loop = EventLoop::try_new().unwrap();

    let display_handle = init_wayland_server(&event_loop.handle());

    if running_under_shell {
        run_x11_client(event_loop, display_handle);
    } else {
        run_drm_backend(event_loop, display_handle);
    };

    Ok(())
}

fn init_wayland_server(loop_handle: &LoopHandle<State>) -> DisplayHandle {
    let display: Display<State> = Display::new().unwrap();
    let display_handle = display.handle();

    let source = ListeningSocketSource::new_auto().unwrap();
    let socket_name = source.socket_name().to_string_lossy().into_owned();

    set_var("WAYLAND_DISPLAY", &socket_name);
    set_var("XDG_SESSION_TYPE", "wayland");
    set_var("GDK_BACKEND", "wayland"); // Force GTK apps to run on Wayland.
    set_var("QT_QPA_PLATFORM", "wayland"); // Force QT apps to run on Wayland.

    loop_handle.insert_source(source, |client_stream, _, data| {
        if let Err(err) = data
            .common.display_handle
            .insert_client(client_stream, Arc::new(ClientState::default()))
        {
            warn!("Error adding wayland client: {}", err);
        };
    }).unwrap();


    loop_handle.insert_source(Generic::new(display, Interest::READ, Mode::Level), |_, display, data| {
        profiling::scope!("dispatch_clients");
        // Safety: we don't drop the display
        unsafe {
            display.get_mut().dispatch_clients(data).unwrap();
        }
        Ok(PostAction::Continue)
    }).unwrap();

    display_handle
}

pub fn run_event_loop(event_loop: &mut EventLoop<State>, state: &mut State) {
    event_loop.run(None, state, |state: &mut State| {
        if state.common.should_stop {
            info!("Shutting down");
            state.common.loop_signal.stop();
            state.common.loop_signal.wakeup();
            return;
        }
        let _ = state.common.display_handle.flush_clients();
    }).unwrap();
}
