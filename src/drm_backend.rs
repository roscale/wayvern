use std::collections::HashMap;
use std::os::fd::FromRawFd;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use smithay::{
    backend::allocator::gbm::GbmAllocator,
    output::Mode,
    reexports::{
        calloop::EventLoop,
        gbm::BufferObjectFlags as GbmBufferFlags,
        wayland_server::Display,
    },
    wayland::{
        compositor::CompositorState,
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};
use smithay::backend::{egl, SwapBuffersError, vulkan};
use smithay::backend::allocator::{Allocator, Fourcc, Slot, Swapchain};
use smithay::backend::allocator::dmabuf::{AnyError, AsDmabuf, Dmabuf, DmabufAllocator};
use smithay::backend::allocator::gbm::GbmDevice;
use smithay::backend::allocator::vulkan::{ImageUsageFlags, VulkanAllocator};
use smithay::backend::drm::{CreateDrmNodeError, DrmDevice, DrmDeviceFd, DrmError, DrmEvent, DrmNode, DrmSurface, GbmBufferedSurface, NodeType};
use smithay::backend::drm::compositor::DrmCompositor;
use smithay::backend::egl::{EGLContext, EGLDevice, EGLDisplay};
use smithay::backend::input::InputEvent;
use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
use smithay::backend::renderer::{Bind, DebugFlags, ExportMem, ImportDma, Offscreen, Renderer};
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::backend::renderer::element::{Kind, RenderElement};
use smithay::backend::renderer::element::texture::{TextureBuffer, TextureRenderElement};
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::backend::session::{libseat, Session};
use smithay::backend::session::libseat::LibSeatSession;
use smithay::backend::udev::{all_gpus, primary_gpu, UdevBackend, UdevEvent};
use smithay::backend::vulkan::version::Version;
use smithay::desktop::utils::OutputPresentationFeedback;
use smithay::output::{Output, PhysicalProperties, Subpixel};
use smithay::reexports::ash::vk::ExtPhysicalDeviceDrmFn;
use smithay::reexports::calloop::{channel, RegistrationToken};
use smithay::reexports::drm::control::{connector, crtc, Device, ModeTypeFlags, OFlag};
use smithay::reexports::drm::Device as _;
use smithay::reexports::input::Libinput;
use smithay::reexports::wayland_server::backend::GlobalId;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::utils::{DeviceFd, Physical, Point, Rectangle, Transform};
use smithay::wayland::drm_lease::DrmLease;
use tracing::{error, info, warn};
use smithay::backend::renderer::damage::Error as OutputDamageTrackerError;
use smithay::backend::renderer::sync::SyncPoint;
use smithay_drm_extras::drm_scanner::{DrmScanEvent, DrmScanner};
use smithay_drm_extras::edid::EdidInfo;

use crate::{App, Backend, CalloopData, flutter_engine::{EmbedderChannels, FlutterEngine}, flutter_engine};

// we cannot simply pick the first supported format of the intersection of *all* formats, because:
// - we do not want something like Abgr4444, which looses color information, if something better is available
// - some formats might perform terribly
// - we might need some work-arounds, if one supports modifiers, but the other does not
//
// So lets just pick `ARGB2101010` (10-bit) or `ARGB8888` (8-bit) for now, they are widely supported.
const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr2101010,
    Fourcc::Argb2101010,
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
];
const SUPPORTED_FORMATS_8BIT_ONLY: &[Fourcc] = &[Fourcc::Abgr8888, Fourcc::Argb8888];

pub fn run_drm_backend() {
    let mut event_loop = EventLoop::try_new().unwrap();
    let display: Display<App<DrmBackend>> = Display::new().unwrap();
    let mut display_handle = display.handle();

    /*
     * Initialize session
     */
    let (session, notifier) = match LibSeatSession::new() {
        Ok(ret) => ret,
        Err(err) => {
            error!("Could not initialize a session: {}", err);
            return;
        }
    };

    /*
     * Initialize the compositor
     */
    let primary_gpu = if let Ok(var) = std::env::var("ANVIL_DRM_DEVICE") {
        DrmNode::from_path(var).expect("Invalid drm device path")
    } else {
        primary_gpu(&session.seat())
            .unwrap()
            .and_then(|x| DrmNode::from_path(x).ok()?.node_with_type(NodeType::Render)?.ok())
            .unwrap_or_else(|| {
                all_gpus(session.seat())
                    .unwrap()
                    .into_iter()
                    .find_map(|x| DrmNode::from_path(x).ok())
                    .expect("No GPU!")
            })
    };
    info!("Using {} as primary gpu.", primary_gpu);

    /*
     * Initialize the udev backend
     */
    let udev_backend = match UdevBackend::new(&session.seat()) {
        Ok(ret) => ret,
        Err(err) => {
            error!(error = ?err, "Failed to initialize udev backend");
            return;
        }
    };

    /*
     * Initialize libinput backend
     */
    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        session.clone().into(),
    );
    libinput_context.udev_assign_seat(&session.seat()).unwrap();
    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    let mut state = App {
        running: Arc::new(AtomicBool::new(true)),
        display_handle: display.handle(),
        loop_handle: event_loop.handle(),
        backend_data: DrmBackend {
            session,
            display_handle: display.handle(),
            backends: HashMap::new(),
            primary_gpu: primary_gpu.clone(),
            swapchain: None,
            current_slot: None,
        },
        // flutter_engine: FlutterEngine::new(&egl_context).unwrap(),
        flutter_engine: FlutterEngine::new(),
        mouse_button_tracker: Default::default(),
        mouse_position: Default::default(),
        compositor_state: CompositorState::new::<App<DrmBackend>>(&display_handle),
        xdg_shell_state: XdgShellState::new::<App<DrmBackend>>(&display_handle),
        shm_state: ShmState::new::<App<DrmBackend>>(&display_handle, vec![]),
    };

    state.device_added(primary_gpu, &primary_gpu.dev_path().unwrap()).unwrap();

    // Mandatory formats by the Wayland spec.
    // TODO: Add more formats based on the GLES version.
    state.shm_state.update_formats([
        wl_shm::Format::Argb8888,
        wl_shm::Format::Xrgb8888,
    ]);

    let skip_vulkan = std::env::var("ANVIL_NO_VULKAN")
        .map(|x| {
            x == "1" || x.to_lowercase() == "true" || x.to_lowercase() == "yes" || x.to_lowercase() == "y"
        })
        .unwrap_or(false);

    let vulkan_allocator = if !skip_vulkan {
        vulkan::Instance::new(Version::VERSION_1_2, None)
            .ok()
            .and_then(|instance| {
                vulkan::PhysicalDevice::enumerate(&instance).ok().and_then(|devices| {
                    devices
                        .filter(|phd| phd.has_device_extension(ExtPhysicalDeviceDrmFn::name()))
                        .find(|phd| {
                            phd.primary_node().unwrap() == Some(primary_gpu)
                                || phd.render_node().unwrap() == Some(primary_gpu)
                        })
                })
            })
            .and_then(|physical_device| {
                VulkanAllocator::new(
                    &physical_device,
                    ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED,
                ).ok()
            })
    } else {
        None
    };

    let allocator: Box<dyn Allocator<Buffer=Dmabuf, Error=AnyError>> = match vulkan_allocator {
        None => {
            info!("No vulkan allocator found, using GBM.");
            let gbm_device = state.backend_data.backends.get(&primary_gpu).unwrap().gbm.clone();
            let gbm_allocator = GbmAllocator::new(gbm_device, GbmBufferFlags::RENDERING);
            Box::new(DmabufAllocator(gbm_allocator))
        }
        Some(vulkan_allocator) => {
            Box::new(DmabufAllocator(vulkan_allocator))
        }
    };

    event_loop
        .handle()
        .insert_source(libinput_backend, move |event, _, data| {
            let dh = data.state.backend_data.display_handle.clone();
            match event {
                InputEvent::PointerButton { .. } => {
                    data.state.running.store(false, Ordering::SeqCst);
                }
                InputEvent::Keyboard { .. } => {
                    data.state.running.store(false, Ordering::SeqCst);
                }
                _ => {}
            }
        })
        .unwrap();

    event_loop
        .handle()
        .insert_source(udev_backend, move |event, _, data| match event {
            // UdevEvent::Added { device_id, path } => {
            //     if let Err(err) = DrmNode::from_dev_id(device_id)
            //         .map_err(DeviceAddError::DrmNode)
            //         .and_then(|node| data.state.device_added(node, &path))
            //     {
            //         error!("Skipping device {device_id}: {err}");
            //     }
            // }
            UdevEvent::Changed { device_id } => {
                if let Ok(node) = DrmNode::from_dev_id(device_id) {
                    data.state.device_changed(node)
                }
            }
            // UdevEvent::Removed { device_id } => {
            //     if let Ok(node) = DrmNode::from_dev_id(device_id) {
            //         data.state.device_removed(node)
            //     }
            // }
            _ => {}
        })
        .unwrap();


    let mut primary_gpu_backend = state.backend_data.backends.get_mut(&primary_gpu).unwrap();

    // let gbm_device = primary_gpu_backend.gbm.clone();

    let EmbedderChannels {
        rx_present,
        rx_request_fbo: rx_request_fbo,
        tx_fbo: mut tx_fbo,
        tx_output_height,
        rx_baton: _,
    } = state.flutter_engine.run(&primary_gpu_backend.gles_renderer.egl_context()).unwrap();

    state.flutter_engine.send_window_metrics((2560, 1440).into()).unwrap();

    // let size = window.size();
    // tx_output_height.send(size.h).unwrap();
    // flutter_engine.send_window_metrics((size.w as u32, size.h as u32).into()).unwrap();

    // Mandatory formats by the Wayland spec.
    // TODO: Add more formats based on the GLES version.
    state.shm_state.update_formats([
        wl_shm::Format::Argb8888,
        wl_shm::Format::Xrgb8888,
    ]);

    let mut baton = None;

    // let buf: GlesTexture = primary_gpu_backend.gles_renderer.create_buffer(Fourcc::Argb8888, (1000, 1000).into()).unwrap();

    let mut gles_renderer = &mut state.backend_data.backends.get_mut(&primary_gpu).unwrap().gles_renderer;
    let modifiers = gles_renderer.egl_context().dmabuf_texture_formats().iter().map(|format| format.modifier).collect::<Vec<_>>();
    let mut swapchain = Swapchain::new(allocator, 2560, 1440, Fourcc::Argb8888, modifiers);

    state.backend_data.swapchain = Some(swapchain);


    // Swapchain<Box<dyn Allocator>>

    // let swapchain = primary_gpu_backend.gles_renderer.create_swapchain((2560, 1440).into()).unwrap();



    event_loop.handle().insert_source(rx_request_fbo, move |_, _, data| {
        let mut primary_gpu = &mut data.state.backend_data.primary_gpu;
        let backend = data.state.backend_data.backends.get_mut(&primary_gpu).unwrap();
        let first_crtc = backend.surfaces.keys().next().unwrap().clone();
        let mut surface = backend.surfaces.get_mut(&first_crtc).unwrap();

        match &mut surface.compositor {
            SurfaceComposition::Surface { ref mut surface, .. } => {
                let (dmabuf, _age) = surface.next_buffer().unwrap();
                data.tx_fbo.send(Some(dmabuf)).unwrap();
            }
            SurfaceComposition::Compositor(_) => {
                let slot = data.state.backend_data.swapchain.as_mut().unwrap().acquire().ok().flatten().unwrap();
                let dmabuf = slot.export().unwrap();
                data.state.backend_data.current_slot = Some(slot);
                data.tx_fbo.send(Some(dmabuf)).unwrap();
            }
        }

        // match data.state.backend_data.backends.get(&primary_gpu).unwrap().surfaces

        // match data.state.backend_data.compositor_state.buffer() {
        //     Ok((dmabuf, _age)) => {
        //         let _ = data.tx_fbo.send(Some(dmabuf));
        //     }
        //     Err(err) => {
        //         error!("{err}");
        //         let _ = data.tx_fbo.send(None);
        //     }
        // }
    }).unwrap();

    event_loop.handle().insert_source(rx_present, move |_, _, data| {
        let primary_gpu = data.state.backend_data.primary_gpu;
        let mut primary_gpu_backend = data.state.backend_data.backends.get_mut(&primary_gpu).unwrap();
        let first_crtc = primary_gpu_backend.surfaces.keys().next().unwrap().clone();
        let mut surface = primary_gpu_backend.surfaces.get_mut(&first_crtc).unwrap();

        match &mut surface.compositor {
            SurfaceComposition::Surface { ref mut surface, .. } => {
                surface.queue_buffer(None, None, None).unwrap();
            }
            SurfaceComposition::Compositor(ref mut compositor) => {
                let mut gles_renderer = &mut primary_gpu_backend.gles_renderer;

                let slot = data.state.backend_data.current_slot.take().unwrap();
                let texture = gles_renderer.import_dmabuf(&slot.export().unwrap(), None).unwrap();

                let texture_buffer = TextureBuffer::from_texture(gles_renderer, texture, 1, Transform::Flipped180, None);
                let texture_render_element = TextureRenderElement::from_texture_buffer(
                    Point::from((0.0, 0.0)),
                    &texture_buffer,
                    None,
                    None,
                    None,
                    Kind::Unspecified,
                );
                compositor.render_frame::<GlesRenderer, TextureRenderElement<GlesTexture>, GlesTexture>(gles_renderer, &[texture_render_element], [0.0, 0.0, 0.0, 0.0]).unwrap();
                compositor.queue_frame(None).unwrap();
                data.state.backend_data.swapchain.as_mut().unwrap().submitted(&slot);
            }
        }

        // let primary_handle = compositor.surface().planes().primary.handle;


        // if let Err(err) = data.state.backend_data.x11_surface.submit() {
        //     data.state.backend_data.x11_surface.reset_buffers();
        //     warn!("Failed to submit buffer: {}. Retrying", err);
        // };
    }).unwrap();

    while state.running.load(Ordering::SeqCst) {
        let mut calloop_data = CalloopData {
            state,
            display_handle,
            tx_fbo,
            baton,
        };

        let result = event_loop.dispatch(None, &mut calloop_data);

        CalloopData {
            state,
            display_handle,
            tx_fbo,
            baton,
        } = calloop_data;

        if result.is_err() {
            state.running.store(false, Ordering::SeqCst);
        } else {
            display_handle.flush_clients().unwrap();
        }
    }

    // Avoid indefinite hang in the Flutter render thread waiting for new rbo.
    drop(tx_fbo);
}

pub struct DrmBackend {
    pub session: LibSeatSession,
    display_handle: DisplayHandle,
    backends: HashMap<DrmNode, BackendData>,
    primary_gpu: DrmNode,
    swapchain: Option<Swapchain<Box<dyn Allocator<Buffer=Dmabuf, Error=AnyError> + 'static>>>,
    current_slot: Option<Slot<Dmabuf>>,
}

impl Backend for DrmBackend {}

struct BackendData {
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    non_desktop_connectors: Vec<(connector::Handle, crtc::Handle)>,
    // leasing_global: Option<DrmLeaseState>,
    active_leases: Vec<DrmLease>,
    gbm: GbmDevice<DrmDeviceFd>,
    drm: DrmDevice,
    drm_scanner: DrmScanner,
    render_node: DrmNode,
    registration_token: RegistrationToken,
    gles_renderer: GlesRenderer,
}

#[derive(Debug, thiserror::Error)]
enum DeviceAddError {
    #[error("Failed to open device using libseat: {0}")]
    DeviceOpen(libseat::Error),
    #[error("Failed to initialize drm device: {0}")]
    DrmDevice(DrmError),
    #[error("Failed to initialize gbm device: {0}")]
    GbmDevice(std::io::Error),
    #[error("Failed to access drm node: {0}")]
    DrmNode(CreateDrmNodeError),
    #[error("Failed to add device to GpuManager: {0}")]
    AddNode(egl::Error),
}

impl App<DrmBackend> {
    fn device_added(&mut self, node: DrmNode, path: &Path) -> Result<(), DeviceAddError> {
        // Try to open the device
        let fd = self
            .backend_data
            .session
            .open(
                path,
                OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NOCTTY | OFlag::O_NONBLOCK,
            )
            .map_err(DeviceAddError::DeviceOpen)?;

        let fd = DrmDeviceFd::new(unsafe { DeviceFd::from_raw_fd(fd) });

        let (drm, notifier) = DrmDevice::new(fd.clone(), true).map_err(DeviceAddError::DrmDevice)?;
        let gbm = GbmDevice::new(fd).map_err(DeviceAddError::GbmDevice)?;

        let registration_token = self
            .loop_handle
            .insert_source(
                notifier,
                move |event, metadata, data: &mut CalloopData<_>| match event {
                    DrmEvent::VBlank(crtc) => {
                        data.state.backend_data.backends.get_mut(&node).unwrap().surfaces.get_mut(&crtc).unwrap().compositor.frame_submitted();
                        // compositor.frame_submitted();

                        if let Some(baton) = data.baton.take() {

                            // self.backend_data.swapchain.unwrap().submitted();
                            data.state.flutter_engine.on_vsync(baton).unwrap();
                            data.baton = None;
                        }

                        dbg!("vblank");
                        profiling::scope!("vblank", &format!("{crtc:?}"));
                        // TODO
                    }
                    DrmEvent::Error(error) => {
                        error!("{:?}", error);
                    }
                },
            )
            .unwrap();

        let egl_display = EGLDisplay::new(gbm.clone()).expect("Failed to create EGLDisplay");
        let render_node = EGLDevice::device_for_display(&egl_display)
            .ok()
            .and_then(|x| x.try_get_render_node().ok().flatten())
            .unwrap_or(node);

        // self.backend_data
        //     .gpus
        //     .as_mut()
        //     .add_node(render_node, gbm.clone())
        //     .map_err(DeviceAddError::AddNode)?;

        self.backend_data.backends.insert(
            node,
            BackendData {
                registration_token,
                gbm,
                drm,
                drm_scanner: DrmScanner::new(),
                non_desktop_connectors: Vec::new(),
                render_node,
                surfaces: HashMap::new(),
                // leasing_global: DrmLeaseState::new::<App<DrmBackend>>(&self.display_handle, &node)
                //     .map_err(|err| {
                //         // TODO replace with inspect_err, once stable
                //         warn!(?err, "Failed to initialize drm lease global for: {}", node);
                //         err
                //     })
                //     .ok(),
                active_leases: Vec::new(),
                gles_renderer: unsafe { GlesRenderer::new(EGLContext::new(&egl_display).unwrap()) }.unwrap(),
            },
        );

        self.device_changed(node);

        Ok(())
    }

    fn connector_connected(&mut self, node: DrmNode, connector: connector::Info, crtc: crtc::Handle) {
        let device = if let Some(device) = self.backend_data.backends.get_mut(&node) {
            device
        } else {
            return;
        };

        // let mut renderer = self
        //     .backend_data
        //     .gpus
        //     .single_renderer(&device.render_node)
        //     .unwrap();
        // let render_formats = renderer.as_mut().egl_context().dmabuf_render_formats().clone();

        let output_name = format!("{}-{}", connector.interface().as_str(), connector.interface_id());
        info!(?crtc, "Trying to setup connector {}", output_name,);

        let non_desktop = device
            .drm
            .get_properties(connector.handle())
            .ok()
            .and_then(|props| {
                let (info, value) = props
                    .into_iter()
                    .filter_map(|(handle, value)| {
                        let info = device.drm.get_property(handle).ok()?;

                        Some((info, value))
                    })
                    .find(|(info, _)| info.name().to_str() == Ok("non-desktop"))?;

                info.value_type().convert_value(value).as_boolean()
            })
            .unwrap_or(false);

        let (make, model) = EdidInfo::for_connector(&device.drm, connector.handle())
            .map(|info| (info.manufacturer, info.model))
            .unwrap_or_else(|| ("Unknown".into(), "Unknown".into()));

        if non_desktop {
            info!("Connector {} is non-desktop, setting up for leasing", output_name);
            device.non_desktop_connectors.push((connector.handle(), crtc));
            // if let Some(lease_state) = device.leasing_global.as_mut() {
            //     lease_state.add_connector::<AnvilState<UdevData>>(
            //         connector.handle(),
            //         output_name,
            //         format!("{} {}", make, model),
            //     );
            // }
        } else {
            let mode_id = connector
                .modes()
                .iter()
                .position(|mode| mode.mode_type().contains(ModeTypeFlags::PREFERRED))
                .unwrap_or(0);

            let drm_mode = connector.modes()[mode_id];
            let wl_mode = Mode::from(drm_mode);

            let surface = match device.drm.create_surface(crtc, drm_mode, &[connector.handle()]) {
                Ok(surface) => surface,
                Err(err) => {
                    warn!("Failed to create drm surface: {}", err);
                    return;
                }
            };

            let (phys_w, phys_h) = connector.size().unwrap_or((0, 0));
            let output = Output::new(
                output_name,
                PhysicalProperties {
                    size: (phys_w as i32, phys_h as i32).into(),
                    subpixel: Subpixel::Unknown,
                    make,
                    model,
                },
            );
            let global = output.create_global::<App<DrmBackend>>(&self.display_handle);

            // let x = self
            //     .space
            //     .outputs()
            //     .fold(0, |acc, o| acc + self.space.output_geometry(o).unwrap().size.w);
            // let position = (x, 0).into();

            let position = (0, 0).into();

            output.set_preferred(wl_mode);
            output.change_current_state(Some(wl_mode), None, None, Some(position));

            // self.space.map_output(&output, position);

            // output.user_data().insert_if_missing(|| UdevOutputId {
            //     crtc,
            //     device_id: node,
            // });

            #[cfg(feature = "debug")]
                let fps_element = self.backend_data.fps_texture.clone().map(FpsElement::new);

            let allocator = GbmAllocator::new(
                device.gbm.clone(),
                GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
            );

            let color_formats = if std::env::var("ANVIL_DISABLE_10BIT").is_ok() {
                SUPPORTED_FORMATS_8BIT_ONLY
            } else {
                SUPPORTED_FORMATS
            };

            let render_formats = device.gles_renderer.egl_context().dmabuf_render_formats().clone();

            let compositor = if std::env::var("ANVIL_DISABLE_DRM_COMPOSITOR").is_ok() {
                let gbm_surface =
                    match GbmBufferedSurface::new(surface, allocator, color_formats, render_formats) {
                        Ok(renderer) => renderer,
                        Err(err) => {
                            warn!("Failed to create rendering surface: {}", err);
                            return;
                        }
                    };
                SurfaceComposition::Surface {
                    surface: gbm_surface,
                    damage_tracker: OutputDamageTracker::from_output(&output),
                    debug_flags: DebugFlags::empty(),
                }
            } else {
                let driver = match device.drm.get_driver() {
                    Ok(driver) => driver,
                    Err(err) => {
                        warn!("Failed to query drm driver: {}", err);
                        return;
                    }
                };

                let mut planes = surface.planes().clone();

                // Using an overlay plane on a nvidia card breaks
                if driver.name().to_string_lossy().to_lowercase().contains("nvidia")
                    || driver
                    .description()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains("nvidia")
                {
                    planes.overlay = vec![];
                }

                let mut compositor = match DrmCompositor::new(
                    &output,
                    surface,
                    Some(planes),
                    allocator,
                    device.gbm.clone(),
                    color_formats,
                    render_formats,
                    device.drm.cursor_size(),
                    Some(device.gbm.clone()),
                ) {
                    Ok(compositor) => compositor,
                    Err(err) => {
                        warn!("Failed to create drm compositor: {}", err);
                        return;
                    }
                };
                compositor.set_debug_flags(DebugFlags::empty());
                SurfaceComposition::Compositor(compositor)
            };

            // let dmabuf_feedback = get_surface_dmabuf_feedback(
            //     self.backend_data.primary_gpu,
            //     device.render_node,
            //     &mut self.backend_data.gpus,
            //     &compositor,
            // );
            //
            let mut surface = SurfaceData {
                dh: self.display_handle.clone(),
                device_id: node,
                render_node: device.render_node,
                global: Some(global),
                compositor,
                #[cfg(feature = "debug")]
                fps: fps_ticker::Fps::default(),
                #[cfg(feature = "debug")]
                fps_element,
                // dmabuf_feedback,
            };

            pub static CLEAR_COLOR: [f32; 4] = [0.8, 0.8, 0.9, 1.0];

            let renderer = &mut device.gles_renderer;

            surface
                .compositor
                .render_frame::<_, TextureRenderElement<_>, GlesTexture>(renderer, &[], CLEAR_COLOR).unwrap();
            surface.compositor.queue_frame(None, None, None).unwrap();
            surface.compositor.reset_buffers();

            device.surfaces.insert(crtc, surface);

            // self.schedule_initial_render(node, crtc, self.handle.clone());
        }
    }

    fn connector_disconnected(&mut self, node: DrmNode, connector: connector::Info, crtc: crtc::Handle) {
        let device = if let Some(device) = self.backend_data.backends.get_mut(&node) {
            device
        } else {
            return;
        };

        if let Some(pos) = device
            .non_desktop_connectors
            .iter()
            .position(|(handle, _)| *handle == connector.handle())
        {
            let _ = device.non_desktop_connectors.remove(pos);
            // if let Some(leasing_state) = device.leasing_global.as_mut() {
            //     leasing_state.withdraw_connector(connector.handle());
            // }
        } else {
            device.surfaces.remove(&crtc);

            // let output = self
            //     .space
            //     .outputs()
            //     .find(|o| {
            //         o.user_data()
            //             .get::<UdevOutputId>()
            //             .map(|id| id.device_id == node && id.crtc == crtc)
            //             .unwrap_or(false)
            //     })
            //     .cloned();
            //
            // if let Some(output) = output {
            //     self.space.unmap_output(&output);
            // }
        }
    }

    fn device_changed(&mut self, node: DrmNode) {
        let device = if let Some(device) = self.backend_data.backends.get_mut(&node) {
            device
        } else {
            return;
        };

        for event in device.drm_scanner.scan_connectors(&device.drm) {
            match event {
                DrmScanEvent::Connected {
                    connector,
                    crtc: Some(crtc),
                } => {
                    self.connector_connected(node, connector, crtc);
                }
                DrmScanEvent::Disconnected {
                    connector,
                    crtc: Some(crtc),
                } => {
                    self.connector_disconnected(node, connector, crtc);
                }
                _ => {}
            }
        }

        // fixup window coordinates
        // crate::shell::fixup_positions(&mut self.space, self.pointer.current_location());
    }

    // fn device_removed(&mut self, node: DrmNode) {
    //     let device = if let Some(device) = self.backend_data.backends.get_mut(&node) {
    //         device
    //     } else {
    //         return;
    //     };
    //
    //     let crtcs: Vec<_> = device
    //         .drm_scanner
    //         .crtcs()
    //         .map(|(info, crtc)| (info.clone(), crtc))
    //         .collect();
    //
    //     for (connector, crtc) in crtcs {
    //         self.connector_disconnected(node, connector, crtc);
    //     }
    //
    //     debug!("Surfaces dropped");
    //
    //     // drop the backends on this side
    //     if let Some(mut backend_data) = self.backend_data.backends.remove(&node) {
    //         if let Some(mut leasing_global) = backend_data.leasing_global.take() {
    //             leasing_global.disable_global::<AnvilState<UdevData>>();
    //         }
    //
    //         self.backend_data
    //             .gpus
    //             .as_mut()
    //             .remove_node(&backend_data.render_node);
    //
    //         self.handle.remove(backend_data.registration_token);
    //
    //         debug!("Dropping device");
    //     }
    //
    //     crate::shell::fixup_positions(&mut self.space, self.pointer.current_location());
    // }
}

pub type RenderSurface = GbmBufferedSurface<GbmAllocator<DrmDeviceFd>, Option<OutputPresentationFeedback>>;

pub type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmDevice<DrmDeviceFd>,
    Option<OutputPresentationFeedback>,
    DrmDeviceFd,
>;

enum SurfaceComposition {
    Surface {
        surface: RenderSurface,
        damage_tracker: OutputDamageTracker,
        debug_flags: DebugFlags,
    },
    Compositor(GbmDrmCompositor),
}

struct SurfaceData {
    dh: DisplayHandle,
    device_id: DrmNode,
    render_node: DrmNode,
    global: Option<GlobalId>,
    compositor: SurfaceComposition,
    #[cfg(feature = "debug")]
    fps: fps_ticker::Fps,
    #[cfg(feature = "debug")]
    fps_element: Option<FpsElement<MultiTexture>>,
    // dmabuf_feedback: Option<DrmSurfaceDmabufFeedback>,
}

impl SurfaceComposition {
    #[profiling::function]
    fn frame_submitted(&mut self) -> Result<Option<Option<OutputPresentationFeedback>>, SwapBuffersError> {
        match self {
            SurfaceComposition::Compositor(c) => c.frame_submitted().map_err(Into::<SwapBuffersError>::into),
            SurfaceComposition::Surface { surface, .. } => {
                surface.frame_submitted().map_err(Into::<SwapBuffersError>::into)
            }
        }
    }

    fn format(&self) -> smithay::reexports::gbm::Format {
        match self {
            SurfaceComposition::Compositor(c) => c.format(),
            SurfaceComposition::Surface { surface, .. } => surface.format(),
        }
    }

    fn surface(&self) -> &DrmSurface {
        match self {
            SurfaceComposition::Compositor(c) => c.surface(),
            SurfaceComposition::Surface { surface, .. } => surface.surface(),
        }
    }

    fn reset_buffers(&mut self) {
        match self {
            SurfaceComposition::Compositor(c) => c.reset_buffers(),
            SurfaceComposition::Surface { surface, .. } => surface.reset_buffers(),
        }
    }

    #[profiling::function]
    fn queue_frame(
        &mut self,
        sync: Option<SyncPoint>,
        damage: Option<Vec<Rectangle<i32, Physical>>>,
        user_data: Option<OutputPresentationFeedback>,
    ) -> Result<(), SwapBuffersError> {
        match self {
            SurfaceComposition::Surface { surface, .. } => surface
                .queue_buffer(sync, damage, user_data)
                .map_err(Into::<SwapBuffersError>::into),
            SurfaceComposition::Compositor(c) => {
                c.queue_frame(user_data).map_err(Into::<SwapBuffersError>::into)
            }
        }
    }

    #[profiling::function]
    fn render_frame<R, E, Target>(
        &mut self,
        renderer: &mut R,
        elements: &[E],
        clear_color: [f32; 4],
    ) -> Result<(), smithay::backend::SwapBuffersError>
        where
            R: Renderer + Bind<Dmabuf> + Bind<Target> + Offscreen<Target> + ExportMem,
            <R as Renderer>::TextureId: 'static,
            <R as Renderer>::Error: Into<SwapBuffersError>,
            E: RenderElement<R>,
    {
        match self {
            SurfaceComposition::Surface {
                surface,
                damage_tracker,
                debug_flags,
            } => {
                let (dmabuf, age) = surface.next_buffer().map_err(Into::<SwapBuffersError>::into)?;
                renderer.bind(dmabuf).map_err(Into::<SwapBuffersError>::into)?;
                let current_debug_flags = renderer.debug_flags();
                renderer.set_debug_flags(*debug_flags);
                let res = damage_tracker
                    .render_output(renderer, age.into(), elements, clear_color)
                    .map(|res| {
                        #[cfg(feature = "renderer_sync")]
                        res.sync.wait();
                        let rendered = res.damage.is_some();
                        ()
                    })
                    .map_err(|err| match err {
                        OutputDamageTrackerError::Rendering(err) => err.into(),
                        _ => unreachable!(),
                    });
                renderer.set_debug_flags(current_debug_flags);
                res
            }
            SurfaceComposition::Compositor(compositor) => compositor
                .render_frame(renderer, elements, clear_color)
                .map(|render_frame_result| {
                    #[cfg(feature = "renderer_sync")]
                    if let PrimaryPlaneElement::Swapchain(element) = render_frame_result.primary_element {
                        element.sync.wait();
                    }
                    ()
                })
                .map_err(|err| match err {
                    smithay::backend::drm::compositor::RenderFrameError::PrepareFrame(err) => err.into(),
                    smithay::backend::drm::compositor::RenderFrameError::RenderFrame(
                        OutputDamageTrackerError::Rendering(err),
                    ) => err.into(),
                    _ => unreachable!(),
                }),
        }
    }

    fn set_debug_flags(&mut self, flags: DebugFlags) {
        match self {
            SurfaceComposition::Surface {
                surface, debug_flags, ..
            } => {
                *debug_flags = flags;
                surface.reset_buffers();
            }
            SurfaceComposition::Compositor(c) => c.set_debug_flags(flags),
        }
    }
}
