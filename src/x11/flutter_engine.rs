use crate::{
    flutter_engine_rs::{Engine, EngineCallbacks},
    state::Flutter,
};
use anyhow::{Context, Result};
use smithay::{
    backend::{
        egl::{EGLContext, get_proc_address},
        renderer::{Bind, ImportDma, Renderer, Texture},
    },
    reexports::calloop::{
        self, LoopHandle, channel,
        timer::{TimeoutAction, Timer},
    },
    utils::{Rectangle, Transform},
};
use std::{
    ffi::c_void,
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};
use tracing::error;

use crate::state::State;

pub struct UserData {
    proc_resolver: unsafe fn(&str) -> *const c_void,
    gl_finish: unsafe extern "system" fn(),
    flutter_main_context: EGLContext,
    flutter_resource_context: EGLContext,
    tx_request_texture: calloop::channel::Sender<(i64, u32, u32, mpsc::Sender<Option<u32>>)>,
    tx_external_texture: calloop::channel::Sender<(i64, mpsc::Sender<Option<u32>>)>,
    tx_present: calloop::channel::Sender<i64>,
}

pub fn start_flutter_engine(loop_handle: LoopHandle<State>, state: &mut State) -> Result<Flutter> {
    let x11_backend = state.backend.x11();

    let egl_display = &x11_backend.egl_display;
    let egl_context = x11_backend.renderer.egl_context();

    let flutter_main_context = EGLContext::new_shared(egl_display, egl_context)
        .with_context(|| "Failed to create shared EGL context")?;

    let flutter_resource_context = EGLContext::new_shared(egl_display, egl_context)
        .with_context(|| "Failed to create shared EGL context")?;

    let pending_baton = Arc::new(Mutex::new(None));

    let (tx_request_texture, rx_request_texture) =
        calloop::channel::channel::<(i64, u32, u32, mpsc::Sender<Option<u32>>)>();

    let (tx_external_texture, rx_external_texture) =
        calloop::channel::channel::<(i64, mpsc::Sender<Option<u32>>)>();

    let (tx_present, rx_present) = calloop::channel::channel::<i64>();

    let engine = Engine::<UserData>::new(
        &state.common.flutter_project,
        !cfg!(debug_assertions),
        EngineCallbacks {
            proc_resolver: Box::new(|name, user_data| unsafe { (user_data.proc_resolver)(name) }),
            external_texture_frame: Box::new(|texture_id, user_data| {
                let (tx_texture, rx_texture) = mpsc::channel();

                if let Err(err) = user_data.tx_external_texture.send((texture_id, tx_texture)) {
                    error!("Could not request texture name for texture_id {texture_id}, {err}");
                    return None;
                }

                rx_texture
                    .recv()
                    .inspect_err(|err| error!("Could not receive texture name, {err}"))
                    .ok()
                    .flatten()
            }),
            make_current: Box::new(|user_data| unsafe {
                user_data.flutter_main_context.make_current().is_ok()
            }),
            clear_current: Box::new(|user_data| user_data.flutter_main_context.unbind().is_ok()),
            make_resource_current: Box::new(|user_data| unsafe {
                user_data.flutter_resource_context.make_current().is_ok()
            }),
            request_texture: Box::new(|view_id, w, h, user_data| {
                let (tx_texture, rx_texture) = mpsc::channel();

                if let Err(err) = user_data
                    .tx_request_texture
                    .send((view_id, w, h, tx_texture))
                {
                    error!("Could not request texture for view {view_id}, {err}");
                    return None;
                }

                match rx_texture.recv() {
                    Ok(texture) => texture,
                    Err(err) => {
                        error!("Could not receive texture name, {err}");
                        None
                    }
                }
            }),
            vsync: Box::new({
                let pending_baton = pending_baton.clone();
                move |baton, _user_data| *pending_baton.lock().unwrap() = Some(baton)
            }),
            present: Box::new(|view_id, user_data| {
                unsafe { (user_data.gl_finish)() };

                if let Err(err) = user_data.tx_present.send(view_id) {
                    error!("Failed to send view_id {view_id} to tx_present, {err}");
                }
            }),
        },
        UserData {
            proc_resolver: get_proc_address,
            tx_external_texture,
            gl_finish: unsafe {
                let gl_finish = get_proc_address("glFinish");
                std::mem::transmute::<*const std::ffi::c_void, unsafe extern "system" fn()>(
                    gl_finish,
                )
            },
            flutter_main_context,
            flutter_resource_context,
            tx_request_texture,
            tx_present,
        },
    )
    .with_context(|| "Could not create Engine")?;

    loop_handle
        .insert_source(rx_request_texture, |event, _, state| {
            let channel::Event::Msg((view_id, width, height, tx_texture)) = event else {
                error!("tx_request_texture channel closed");
                return;
            };

            let x11 = state.backend.x11();

            let Some(surface) = x11.surfaces.iter_mut().find(|s| s.view_id == view_id) else {
                error!("No X11 surface found with view id {view_id}");

                if let Err(err) = tx_texture.send(None) {
                    error!("Failed to send texture id None to tx_texture, {err}");
                    return;
                }

                return;
            };

            surface.swapchain.resize(width, height);

            let slot = match surface.swapchain.acquire() {
                Ok(Some(buffer)) => buffer,
                Ok(None) => {
                    error!("No swapchain slots available");

                    if let Err(err) = tx_texture.send(None) {
                        error!("Failed to send texture id None to tx_texture, {err}");
                        return;
                    }

                    return;
                }
                Err(err) => {
                    error!("{err:?}");

                    if let Err(err) = tx_texture.send(None) {
                        error!("Failed to send texture id None to tx_texture, {err}");
                        return;
                    }

                    return;
                }
            };

            let gl_texture = match x11.renderer.import_dmabuf(&slot, None) {
                Ok(gl_texture) => gl_texture,
                Err(err) => {
                    error!("Can't import dmabuf: {err:?}");

                    if let Err(err) = tx_texture.send(None) {
                        error!("Failed to send texture id None to tx_texture, {err}");
                        return;
                    }

                    return;
                }
            };

            let id = gl_texture.tex_id();
            if let Err(err) = tx_texture.send(Some(id)) {
                error!("Failed to send texture id {id} to tx_texture, {err}");
                return;
            }

            surface.slot = Some(slot);
        })
        .unwrap();

    loop_handle
        .insert_source(rx_present, move |event, _, state| {
            let channel::Event::Msg(view_id) = event else {
                error!("tx_present channel closed");
                return;
            };

            let x11 = state.backend.x11();

            let Some(surface) = x11.surfaces.iter_mut().find(|s| s.view_id == view_id) else {
                error!("No X11 surface found with view id {view_id}");
                return;
            };

            let Some(slot) = surface.slot.take() else {
                error!("Present requested without buffer");
                return;
            };

            let texture = match x11.renderer.import_dmabuf(&slot, None) {
                Ok(texture) => texture,
                Err(err) => {
                    error!("Failed to import dmabuf: {err}");
                    return;
                }
            };
            let size = texture.size();
            let physical_size = (size.w, size.h).into();

            let mut x11_buffer = match surface.x11_surface.buffer() {
                Ok((x11_buffer, _age)) => x11_buffer,
                Err(err) => {
                    error!("Failed acquiring X11 surface buffer: {err}");
                    return;
                }
            };

            let mut fb = match x11.renderer.bind(&mut x11_buffer) {
                Ok(fb) => fb,
                Err(err) => {
                    error!("Failed to bind buffer: {err}");
                    surface.x11_surface.reset_buffers();
                    return;
                }
            };

            let mut frame = match x11
                .renderer
                .render(&mut fb, physical_size, Transform::Normal)
            {
                Ok(frame) => frame,
                Err(err) => {
                    error!("Failed to render: {err}");
                    surface.x11_surface.reset_buffers();
                    return;
                }
            };

            if let Err(err) = frame.render_texture_from_to(
                &texture,
                Rectangle::from_size(size.to_f64()),
                Rectangle::from_size(physical_size),
                &[Rectangle::from_size(physical_size)],
                &[Rectangle::from_size(physical_size)],
                Transform::Flipped180,
                1.0,
                None,
                &[],
            ) {
                error!("Failed to render texture: {err}");
                surface.x11_surface.reset_buffers();
                return;
            }

            if let Err(err) = surface.x11_surface.submit() {
                error!("Failed to submit X11 buffer: {err}");
                surface.x11_surface.reset_buffers();
            }

            let output = state.common.output.clone();

            for window in state.common.space.elements() {
                window.send_frame(
                    &output,
                    state.common.clock.now(),
                    Some(Duration::ZERO),
                    |_, _| Some(output.clone()),
                )
            }
        })
        .unwrap();

    loop_handle
        .insert_source(rx_external_texture, move |event, _, state| {
            let channel::Event::Msg((texture_id, texture_name)) = event else {
                error!("tx_external_texture channel closed");
                return;
            };

            let texture = state
                .common
                .textures_by_texture_id
                .get(&texture_id)
                .map(|texture| texture.tex_id());

            if let Err(err) = texture_name.send(texture) {
                error!("Failed to send texture name: {err}");
            }
        })
        .unwrap();

    const _144_HZ_NS: u64 = 1_000_000_000 / 144;
    let mut vsync = Instant::now();

    let timer = Timer::from_duration(Duration::from_nanos(_144_HZ_NS));
    loop_handle
        .insert_source(timer, move |_, _, state| {
            if let Some(baton) = pending_baton.lock().unwrap().take() {
                let engine = &mut state.common.flutter.engine;
                let now = engine.current_time_ns();

                if let Err(err) = engine.on_vsync(baton, now, now + _144_HZ_NS) {
                    error!("engine.on_vsync failed, {err}");
                }
            }

            vsync += Duration::from_nanos(_144_HZ_NS);
            TimeoutAction::ToInstant(vsync)
        })
        .unwrap();

    Ok(Flutter { engine })
}
