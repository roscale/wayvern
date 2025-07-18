use std::os::fd::OwnedFd;

use anyhow::{Context, Result};
use smithay::backend::allocator::dmabuf::AnyError;
use smithay::backend::allocator::{Allocator, Slot};
use smithay::backend::x11::X11Event;
use smithay::reexports::drm::buffer::{DrmFourcc, DrmModifier};
use smithay::{
    backend::{
        allocator::{
            Swapchain,
            dmabuf::{Dmabuf, DmabufAllocator},
            gbm::{GbmAllocator, GbmBufferFlags},
        },
        drm::{DrmDeviceFd, DrmNode},
        egl::{EGLContext, EGLDevice, EGLDisplay},
        renderer::{Bind, ImportDma, gles::GlesRenderer},
        x11::{Window, WindowBuilder, X11Backend, X11Handle, X11Surface},
    },
    reexports::{calloop::LoopHandle, gbm::Device as GbmDevice, wayland_server::DisplayHandle},
    utils::DeviceFd,
    wayland::dmabuf::DmabufFeedbackBuilder,
};
use tracing::{error, info};

use crate::flutter_engine::{Flutter, start_flutter_engine};
use crate::state::{BackendState, State};

pub struct X11State {
    x11_handle: X11Handle,
    allocator: GbmAllocator<DrmDeviceFd>,
    pub egl_display: EGLDisplay,
    pub renderer: GlesRenderer,
    pub surfaces: Vec<Surface>,

    pub flutter: Option<Flutter>,
    pub next_view_id: i64,
}

impl X11State {
    pub fn filtered_modifiers(&self, fourcc: DrmFourcc) -> Vec<DrmModifier> {
        Bind::<Dmabuf>::supported_formats(&self.renderer)
            .unwrap()
            .iter()
            .filter(|format| format.code == fourcc)
            .map(|format| format.modifier)
            .collect()
    }

    pub fn create_dmabuf_allocator(&self) -> DmabufAllocator<GbmAllocator<DrmDeviceFd>> {
        DmabufAllocator(self.allocator.clone())
    }

    pub fn add_window(&mut self) -> Result<()> {
        let window = WindowBuilder::new()
            .title("Wayvern")
            .build(&self.x11_handle)
            .with_context(|| "Failed to create window")?;

        let fourcc = window.format();
        let filtered_modifiers = self.filtered_modifiers(fourcc);

        let x11_surface = self
            .x11_handle
            .create_surface(
                &window,
                self.create_dmabuf_allocator(),
                filtered_modifiers.iter().copied(),
            )
            .with_context(|| "Failed to create surface")?;

        let allocator = self.create_dmabuf_allocator();

        let fourcc = window.format();
        let modifiers = self.filtered_modifiers(fourcc);

        let swapchain = Swapchain::new(
            Box::new(allocator) as Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError> + 'static>,
            // Dummy values, will be resized later.
            1,
            1,
            fourcc,
            modifiers,
        );

        let surface = Surface {
            window,
            x11_surface,
            view_id: {
                let id = self.next_view_id;
                self.next_view_id += 1;
                id
            },
            swapchain,
            slot: None,
        };

        let window_size = surface.window.size();

        self.flutter
            .as_mut()
            .unwrap()
            .engine
            .add_view(
                surface.view_id,
                window_size.w as usize,
                window_size.h as usize,
                |_| {},
            )
            .with_context(|| format!("Failed to add new Flutter view {}", surface.view_id))?;

        self.surfaces.push(surface);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Surface {
    pub window: Window,
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
        .insert_source(backend, move |event, _, state| match event {
            X11Event::CloseRequested { .. } => {
                state.common.should_stop = true;
            }
            X11Event::Resized {
                new_size: size,
                window_id,
            } => {
                let x11 = state.backend.x11();
                for surface in x11.surfaces.iter() {
                    if surface.window.id() == window_id {
                        if let Err(err) = x11.flutter.as_mut().unwrap().engine.send_view_metrics(
                            surface.view_id,
                            size.w as usize,
                            size.h as usize,
                        ) {
                            error!(
                                "Failed to send metrics to view {}, w: {}, h: {}, {}",
                                surface.view_id, size.w, size.h, err,
                            );
                        }

                        break;
                    }
                }
            }
            _ => {}
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

    // Create the OpenGL context
    let egl_context =
        EGLContext::new(&egl_display).with_context(|| "Failed to create EGL context")?;

    let mut renderer = unsafe { GlesRenderer::new(egl_context) }
        .with_context(|| "Failed to initialize renderer")?;

    init_egl_client_side(display_handle, state, drm_node, &mut renderer)?;

    state.backend = BackendState::X11(Box::new(X11State {
        x11_handle,
        allocator: try_gbm_allocator(fd).context("Failed to create allocator for x11")?,
        egl_display,
        renderer,
        surfaces: vec![],

        flutter: None,
        next_view_id: 0,
    }));

    let flutter = start_flutter_engine(loop_handle.clone(), state)
        .with_context(|| "Failed to start Flutter engine")?;

    state.backend.x11().flutter = Some(flutter);

    state
        .backend
        .x11()
        .add_window()
        .with_context(|| "Failed to create wl_output")?;

    Ok(())
}

fn init_egl_client_side<R>(
    dh: &DisplayHandle,
    state: &mut State,
    render_node: DrmNode,
    renderer: &mut R,
) -> Result<()>
where
    R: ImportDma,
{
    let default_feedback =
        DmabufFeedbackBuilder::new(render_node.dev_id(), renderer.dmabuf_formats())
            .build()
            .unwrap();

    let _dmabuf_global = state
        .common
        .dmabuf_state
        .create_global_with_default_feedback::<State>(dh, &default_feedback);

    info!("EGL hardware-acceleration enabled.");

    Ok(())
}

fn try_gbm_allocator(fd: OwnedFd) -> Option<GbmAllocator<DrmDeviceFd>> {
    // Create the gbm device for buffer allocation.
    let device = match GbmDevice::new(DrmDeviceFd::new(DeviceFd::from(fd))) {
        Ok(gbm) => gbm,
        Err(err) => {
            error!(?err, "Failed to create GBM device.");
            return None;
        }
    };

    Some(GbmAllocator::new(device, GbmBufferFlags::RENDERING))
}
