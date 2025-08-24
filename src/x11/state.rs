use std::{os::fd::OwnedFd, rc::Rc};

use anyhow::{Context, Result};
use smithay::{
    backend::{
        allocator::{
            Allocator, Slot, Swapchain,
            dmabuf::{AnyError, Dmabuf, DmabufAllocator},
            gbm::{GbmAllocator, GbmBufferFlags},
        },
        drm::{DrmDeviceFd, DrmNode},
        egl::{EGLContext, EGLDevice, EGLDisplay},
        renderer::{Bind, ImportDma, ImportEgl, gles::GlesRenderer},
        x11::{Window, WindowBuilder, X11Backend, X11Handle, X11Surface},
    },
    output::Mode,
    reexports::{
        calloop::{LoopHandle, channel},
        drm::buffer::{DrmFourcc, DrmModifier},
        gbm,
        wayland_server::DisplayHandle,
    },
    utils::DeviceFd,
    wayland::dmabuf::DmabufFeedbackBuilder,
};
use tracing::{error, info};

use crate::{
    channel_method_call_handler::ChannelMethodCallHandler,
    flutter_engine_rs::platform_channels::{
        method_channel::MethodChannel, standard_method_codec::StandardMethodCodec,
    },
    state::{BackendState, State},
    x11::{
        channel_events::handle_channel_events, flutter_engine::start_flutter_engine,
        window_events::handle_input_event,
    },
};

pub struct X11State {
    x11_handle: X11Handle,
    allocator: GbmAllocator<DrmDeviceFd>,
    pub egl_display: EGLDisplay,
    pub renderer: GlesRenderer,
    pub surfaces: Vec<Surface>,

    pub next_view_id: i64,
}

#[derive(Debug)]
pub struct Surface {
    pub _window: Window,
    pub x11_surface: X11Surface,
    pub view_id: i64,
    pub swapchain: Swapchain<Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError> + 'static>>,
    pub slot: Option<Slot<Dmabuf>>,
}

pub fn init_x11_backend(
    loop_handle: LoopHandle<State>,
    display_handle: &DisplayHandle,
    state: &mut State,
) -> Result<()> {
    let backend = X11Backend::new().with_context(|| "Failed to initialize X11 backend")?;
    let x11_handle = backend.handle();

    loop_handle
        .insert_source(backend, move |event, _, state| {
            handle_input_event(event, state)
        })
        .expect("Failed to insert X11 Backend into event loop");

    // Obtain the DRM node the X server uses for direct rendering.
    let (drm_node, fd) = x11_handle
        .drm_node()
        .with_context(|| "Could not get DRM node used by X server")?;

    let device = EGLDevice::enumerate()
        .with_context(|| "Failed to enumerate EGL devices")?
        .find(|device| device.try_get_render_node().ok().flatten() == Some(drm_node))
        .with_context(|| format!("Failed to find EGLDevice for node {drm_node}"))?;

    // Initialize EGL.
    let egl_display =
        unsafe { EGLDisplay::new(device.clone()).with_context(|| "Failed to create EGLDisplay")? };

    // Create the OpenGL context.
    let egl_context =
        EGLContext::new(&egl_display).with_context(|| "Failed to create EGL context")?;

    let mut renderer = unsafe { GlesRenderer::new(egl_context) }
        .with_context(|| "Failed to initialize renderer")?;

    if renderer.bind_wl_display(display_handle).is_ok() {
        info!("EGL hardware-acceleration enabled");
    }

    create_dmabuf_global(display_handle, state, drm_node, &mut renderer)?;

    state.backend = BackendState::X11(Box::new(X11State {
        x11_handle,
        allocator: try_gbm_allocator(fd).context("Failed to create allocator for x11")?,
        egl_display,
        renderer,
        surfaces: vec![],

        next_view_id: 0,
    }));

    let flutter = start_flutter_engine(loop_handle.clone(), state)
        .with_context(|| "Failed to start Flutter engine")?;

    state.common.flutter = Some(flutter).into();

    register_flutter_channels(loop_handle, state);

    state
        .add_x11_window()
        .with_context(|| "Failed to create wl_output")?;

    Ok(())
}

fn create_dmabuf_global(
    dh: &DisplayHandle,
    state: &mut State,
    render_node: DrmNode,
    renderer: &mut GlesRenderer,
) -> Result<()> {
    let default_feedback =
        DmabufFeedbackBuilder::new(render_node.dev_id(), renderer.dmabuf_formats())
            .build()
            .unwrap();

    let _dmabuf_global = state
        .common
        .dmabuf_state
        .create_global_with_default_feedback::<State>(dh, &default_feedback);

    Ok(())
}

fn try_gbm_allocator(fd: OwnedFd) -> Option<GbmAllocator<DrmDeviceFd>> {
    // Create the gbm device for buffer allocation.
    let device = match gbm::Device::new(DrmDeviceFd::new(DeviceFd::from(fd))) {
        Ok(gbm) => gbm,
        Err(err) => {
            error!(?err, "Failed to create GBM device.");
            return None;
        }
    };

    Some(GbmAllocator::new(device, GbmBufferFlags::RENDERING))
}

impl State {
    fn add_x11_window(&mut self) -> Result<()> {
        let x11_state = self.backend.x11();

        let window = WindowBuilder::new()
            .title("Wayvern")
            .build(&x11_state.x11_handle)
            .with_context(|| "Failed to create window")?;

        let size = window.size();
        let fourcc = window.format();
        let modifiers = filtered_modifiers(&x11_state.renderer, fourcc);

        let x11_surface = x11_state
            .x11_handle
            .create_surface(
                &window,
                DmabufAllocator(x11_state.allocator.clone()),
                modifiers.iter().copied(),
            )
            .with_context(|| "Failed to create surface")?;

        let swapchain = {
            let allocator = DmabufAllocator(x11_state.allocator.clone());

            Swapchain::new(
                Box::new(allocator)
                    as Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError> + 'static>,
                size.w as u32,
                size.h as u32,
                fourcc,
                modifiers,
            )
        };

        let surface = Surface {
            _window: window,
            x11_surface,
            view_id: {
                let id = x11_state.next_view_id;
                x11_state.next_view_id += 1;
                id
            },
            swapchain,
            slot: None,
        };

        self.common
            .flutter
            .engine
            .add_view(surface.view_id, size.w as usize, size.h as usize, |_| {})
            .with_context(|| format!("Failed to add new Flutter view {}", surface.view_id))?;

        let mode = Mode {
            size: (size.w as i32, size.h as i32).into(),
            refresh: 144,
        };
        self.common.output.set_preferred(mode);
        self.common
            .output
            .change_current_state(Some(mode), None, None, None);

        x11_state.surfaces.push(surface);

        Ok(())
    }
}

fn filtered_modifiers(renderer: &GlesRenderer, fourcc: DrmFourcc) -> Vec<DrmModifier> {
    Bind::<Dmabuf>::supported_formats(renderer)
        .unwrap()
        .iter()
        .filter(|format| format.code == fourcc)
        .map(|format| format.modifier)
        .collect()
}

fn register_flutter_channels(loop_handle: LoopHandle<State>, state: &mut State) {
    let messenger = state.common.flutter.engine.binary_messenger();
    let mut window_channel = MethodChannel::new(
        messenger,
        "wayvern/window".to_string(),
        Rc::new(StandardMethodCodec::new()),
    );

    let rx = window_channel.channel_call_handler();
    loop_handle
        .insert_source(rx, |event, (), state| {
            let channel::Event::Msg((method_call, result)) = event else {
                error!("window channel closed");
                return;
            };

            handle_channel_events(method_call, result, state);
        })
        .unwrap();
}
