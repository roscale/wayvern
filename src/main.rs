use std::{
    env::{self},
    ffi::OsString,
    path::PathBuf,
    sync::Arc,
};

use smithay::{
    reexports::{
        calloop::{EventLoop, Interest, Mode, PostAction, generic::Generic},
        wayland_server::{Display, DisplayHandle},
    },
    wayland::socket::ListeningSocketSource,
};
use tracing::{error, info, warn};

use anyhow::{Context, Result};
use state::State;

use crate::x11::init_x11_backend;
mod flutter_engine;
mod flutter_engine_rs;
mod state;
mod wayland;
mod x11;

fn main() -> Result<()> {
    run("flutter".into())
}

pub fn run(flutter_project: PathBuf) -> Result<()> {
    if let Ok(env_filter) = tracing_subscriber::filter::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let running_under_shell = env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok();

    let mut event_loop = EventLoop::try_new().unwrap();

    let (display_handle, _socket_name) = init_wayland_display(&mut event_loop)?;

    let mut state = State::new(
        display_handle.clone(),
        event_loop.get_signal(),
        flutter_project,
    );

    if running_under_shell {
        init_x11_backend(event_loop.handle(), &display_handle, &mut state)
            .with_context(|| "Failed to init x11 backend")?;
    } else {
        // run_drm_backend(event_loop, display_handle);
    };

    // run the event loop
    event_loop.run(None, &mut state, |state| {
        // shall we shut down?
        if state.common.should_stop {
            info!("Shutting down");
            state.common.event_loop_signal.stop();
            state.common.event_loop_signal.wakeup();
            return;
        }

        // send out events
        let _ = state.common.display_handle.flush_clients();
    })?;

    // drop eventloop & state before logger
    std::mem::drop(event_loop);
    std::mem::drop(state);

    Ok(())
}

fn init_wayland_display(event_loop: &mut EventLoop<State>) -> Result<(DisplayHandle, OsString)> {
    let display = Display::new().unwrap();
    let display_handle = display.handle();

    let source = ListeningSocketSource::new_auto().unwrap();
    let socket_name = source.socket_name().to_os_string();
    info!("Listening on {:?}", socket_name);

    event_loop
        .handle()
        .insert_source(source, |client_stream, _, state| {
            let client_state = state.new_client_state();

            if let Err(err) = state
                .common
                .display_handle
                .insert_client(client_stream, Arc::new(client_state))
            {
                warn!(?err, "Error adding wayland client")
            };
        })
        .with_context(|| "Failed to initialize the wayland socket source.")?;

    event_loop
        .handle()
        .insert_source(
            Generic::new(display, Interest::READ, Mode::Level),
            move |_, display, state| {
                // SAFETY: We don't drop the display
                match unsafe { display.get_mut().dispatch_clients(state) } {
                    Ok(_) => Ok(PostAction::Continue),
                    Err(err) => {
                        error!(?err, "I/O error on the Wayland display");
                        state.common.should_stop = true;
                        Err(err)
                    }
                }
            },
        )
        .with_context(|| "Failed to init the wayland event source.")?;

    Ok((display_handle, socket_name))
}
