use std::env::remove_var;
use log::{error, warn};
use smithay::backend::{egl, vulkan, x11};
use smithay::backend::allocator::dmabuf::DmabufAllocator;
use smithay::backend::allocator::gbm::{GbmAllocator, GbmBufferFlags};
use smithay::backend::allocator::vulkan::{ImageUsageFlags, VulkanAllocator};
use smithay::backend::renderer::gles::ffi::Gles2;
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::vulkan::version::Version;
use smithay::backend::x11::{X11Event, X11Surface};
use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::reexports::ash::vk::EXT_PHYSICAL_DEVICE_DRM_NAME;
use smithay::reexports::calloop::{EventLoop, LoopHandle};
use smithay::reexports::gbm;
use smithay::reexports::wayland_server::DisplayHandle;
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::utils::DeviceFd;
use smithay::wayland::dmabuf::{DmabufFeedbackBuilder, DmabufState};
use crate::run_event_loop;
use crate::flutter_engine::{EmbedderChannels, FlutterEngine};
use crate::backends::Backend;
use crate::input_handling::handle_input;
use crate::common::Common;
use crate::state::State;

pub fn run_x11_client(mut event_loop: EventLoop<'static, State>, display_handle: DisplayHandle) {
    let loop_handle: LoopHandle<State> = event_loop.handle();
    let loop_signal = event_loop.get_signal();

    let x11_backend = x11::X11Backend::new().unwrap();
    let x11_handle = x11_backend.handle();

    let (node, fd) = x11_handle.drm_node().expect("Could not get DRM node used by X server");

    let gbm_device = gbm::Device::new(DeviceFd::from(fd)).expect("Failed to create gbm device");
    let egl_display = unsafe { egl::EGLDisplay::new(gbm_device.clone()) }.expect("Failed to create EGLDisplay");
    let egl_context = egl::EGLContext::new(&egl_display).expect("Failed to create EGLContext");

    let window = x11::WindowBuilder::new()
        .title("Wayvern")
        .build(&x11_handle)
        .expect("Failed to create first window");

    let window_size = window.size();
    let window_size = (window_size.w as i32, window_size.h as i32).into();

    let mode = Mode {
        size: window_size,
        refresh: 144_000,
    };
    let output = Output::new(
        "x11".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Wayvern".into(),
            model: "x11".into(),
        },
    );
    let _global = output.create_global::<State>(&display_handle);
    output.change_current_state(Some(mode), None, None, Some((0, 0).into()));
    output.set_preferred(mode);

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
                        .filter(|phd| phd.has_device_extension(EXT_PHYSICAL_DEVICE_DRM_NAME))
                        .find(|phd| {
                            phd.primary_node().unwrap() == Some(node)
                                || phd.render_node().unwrap() == Some(node)
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

    let x11_surface = match vulkan_allocator {
        // Create the surface for the window.
        Some(vulkan_allocator) => x11_handle
            .create_surface(
                &window,
                DmabufAllocator(vulkan_allocator),
                egl_context
                    .dmabuf_render_formats()
                    .iter()
                    .map(|format| format.modifier),
            ).expect("Failed to create X11 surface"),
        None => {
            let gbm_allocator = GbmAllocator::new(gbm_device, GbmBufferFlags::RENDERING);

            x11_handle
                .create_surface(
                    &window,
                    DmabufAllocator(gbm_allocator),
                    egl_context
                        .dmabuf_render_formats()
                        .iter()
                        .map(|format| format.modifier),
                ).expect("Failed to create X11 surface")
        }
    };

    let dmabuf_formats = egl_context.dmabuf_texture_formats()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let _dmabuf_default_feedback = DmabufFeedbackBuilder::new(node.dev_id(), dmabuf_formats)
        .build()
        .unwrap();

    let dmabuf_state = DmabufState::new();
    // let _dmabuf_global = dmabuf_state.create_global_with_default_feedback::<ServerState<X11Data>>(
    //     &display.handle(),
    //     &dmabuf_default_feedback,
    // );

    let (
        flutter_engine,
        EmbedderChannels {
            rx_present,
            rx_request_fbo,
            tx_fbo,
            rx_baton,
            rx_request_external_texture_name,
            tx_external_texture_name,
        },
    ) = FlutterEngine::new(&egl_context, &loop_handle).unwrap();

    let gles_renderer = unsafe { GlesRenderer::new(egl_context) }.expect("Failed to initialize GLES");
    let gl = Gles2::load_with(|s| unsafe { egl::get_proc_address(s) } as *const _);

    let backend = X11Backend {
        x11_surface,
        mode: Mode {
            size: window_size,
            refresh: 144_000,
        },
    };

    let mut common = Common::new(
        display_handle,
        loop_handle.clone(),
        loop_signal,
        "x11".to_string(),
        dmabuf_state,
        flutter_engine,
        tx_fbo,
        rx_baton,
        rx_request_external_texture_name,
        tx_external_texture_name,
        gles_renderer,
        gl,
    );

    let window_size = (window_size.w as u32, window_size.h as u32).into();
    common.flutter_engine.set_canvas_size(window_size, true).unwrap();

    // Mandatory formats by the Wayland spec.
    // TODO: Add more formats based on the GLES version.
    common.shm_state.update_formats([
        wl_shm::Format::Argb8888,
        wl_shm::Format::Xrgb8888,
    ]);

    loop_handle.insert_source(x11_backend, move |event, _, data: &mut State| match event {
        X11Event::CloseRequested { .. } => {
            data.common.should_stop = true;
        }
        X11Event::Resized { new_size, .. } => {
            data.backend.x11().mode = Mode {
                size: (new_size.w as i32, new_size.h as i32).into(),
                refresh: 144_000,
            };

            output.change_current_state(Some(data.backend.x11().mode), None, None, Some((0, 0).into()));
            output.set_preferred(mode);

            let size = (new_size.w as u32, new_size.h as u32).into();
            data.common.flutter_engine.set_canvas_size(size, true).unwrap();
        }
        X11Event::PresentCompleted { .. } | X11Event::Refresh { .. } => data.common.vsync(),
        X11Event::Input { event, window_id: _ } => handle_input(&event, data),
        X11Event::Focus { focused: false, .. } => data.release_all_keys(),
        X11Event::Focus { .. } => {}
    }).unwrap();

    loop_handle.insert_source(rx_request_fbo, move |_, _, data| {
        match data.backend.x11().x11_surface.buffer() {
            Ok((dmabuf, _age)) => {
                let _ = data.common.tx_fbo.send(Some(dmabuf));
            }
            Err(err) => {
                error!("{err}");
                let _ = data.common.tx_fbo.send(None);
            }
        }
    }).unwrap();

    loop_handle.insert_source(rx_present, move |_, _, data| {
        data.common.is_next_vblank_scheduled = true;
        if let Err(err) = data.backend.x11().x11_surface.submit() {
            data.backend.x11().x11_surface.reset_buffers();
            warn!("Failed to submit buffer: {}. Retrying", err);
        };
    }).unwrap();

    let mut state = State {
        common,
        backend: Backend::X11(backend),
    };

    // Make sure clients won't try to connect to the existing X server.
    remove_var("DISPLAY");

    run_event_loop(&mut event_loop, &mut state);
}

pub struct X11Backend {
    pub x11_surface: X11Surface,
    pub mode: Mode,
}
