use std::rc::Rc;
use log::{error, info, warn};
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
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::gbm;
use smithay::reexports::wayland_server::Display;
use smithay::reexports::wayland_server::protocol::wl_shm;
use smithay::utils::DeviceFd;
use smithay::wayland::dmabuf::{DmabufFeedbackBuilder, DmabufState};
use crate::send_frames_surface_tree;
use crate::flutter_engine::{EmbedderChannels, FlutterEngine};
use platform_channels::encodable_value::EncodableValue;
use platform_channels::method_channel::MethodChannel;
use platform_channels::standard_method_codec::StandardMethodCodec;
use crate::backends::Backend;
use crate::input_handling::handle_input;
use crate::server_state::Common;
use crate::state::State;

pub fn run_x11_client() {
    let mut event_loop = EventLoop::try_new().unwrap();
    let loop_handle = event_loop.handle();

    let display: Display<State> = Display::new().unwrap();
    let display_handle = display.handle();

    let x11_backend = x11::X11Backend::new().expect("Failed to initilize X11 backend");
    let x11_handle = x11_backend.handle();

    let (node, fd) = x11_handle.drm_node().expect("Could not get DRM node used by X server");

    let gbm_device = gbm::Device::new(DeviceFd::from(fd)).expect("Failed to create gbm device");
    let egl_display = unsafe { egl::EGLDisplay::new(gbm_device.clone()) }.expect("Failed to create EGLDisplay");
    let egl_context = egl::EGLContext::new(&egl_display).expect("Failed to create EGLContext");

    let window = x11::WindowBuilder::new()
        .title("Anvil")
        .build(&x11_handle)
        .expect("Failed to create first window");

    let mode = Mode {
        size: (window.size().w as i32, window.size().h as i32).into(),
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
            tx_output_height,
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
            size: {
                let s = window.size();
                (s.w as i32, s.h as i32).into()
            },
            refresh: 144_000,
        },
    };

    let mut common = Common::new(
        display,
        loop_handle,
        event_loop.get_signal(),
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
    
    let codec = Rc::new(StandardMethodCodec::new());

    let tx_platform_message = common.tx_platform_message.take().unwrap();
    let mut platform_method_channel = MethodChannel::<EncodableValue>::new(
        common.flutter_engine.binary_messenger.as_mut().unwrap(),
        "platform".to_string(),
        codec,
    );
    // TODO: Provide a way to specify a channel directly, without registering a callback.
    platform_method_channel.set_method_call_handler(Some(Box::new(move |method_call, result| {
        tx_platform_message.send((method_call, result)).unwrap();
    })));

    let size = window.size();
    tx_output_height.send(size.h).unwrap();
    common.flutter_engine.send_window_metrics((size.w as u32, size.h as u32).into()).unwrap();

    // Mandatory formats by the Wayland spec.
    // TODO: Add more formats based on the GLES version.
    common.shm_state.update_formats([
        wl_shm::Format::Argb8888,
        wl_shm::Format::Xrgb8888,
    ]);

    let mut state = State {
        common,
        backend: Backend::X11(backend),
    };

    event_loop
        .handle()
        .insert_source(x11_backend, move |event, _, data: &mut State| match event {
            X11Event::CloseRequested { .. } => {
                data.common.should_stop = true;
            }
            X11Event::Resized { new_size, .. } => {
                let size = { (new_size.w as i32, new_size.h as i32).into() };

                data.backend.x11().mode = Mode {
                    size,
                    refresh: 144_000,
                };

                output.change_current_state(Some(data.backend.x11().mode), None, None, Some((0, 0).into()));
                output.set_preferred(mode);

                let _ = tx_output_height.send(new_size.h);
                data.common.flutter_engine.send_window_metrics((size.w as u32, size.h as u32).into()).unwrap();
            }
            X11Event::PresentCompleted { .. } | X11Event::Refresh { .. } => {
                data.common.is_next_vblank_scheduled = false;
                if let Some(baton) = data.common.baton.take() {
                    data.common.flutter_engine.on_vsync(baton).unwrap();
                }
                let start_time = std::time::Instant::now();
                for surface in data.common.xdg_shell_state.toplevel_surfaces() {
                    send_frames_surface_tree(surface.wl_surface(), start_time.elapsed().as_millis() as u32);
                }
                for surface in data.common.xdg_popups.values() {
                    send_frames_surface_tree(surface.wl_surface(), start_time.elapsed().as_millis() as u32);
                }
            }
            X11Event::Input { event, window_id: _ } => handle_input(&event, data),
            X11Event::Focus { .. } => {}
        })
        .expect("Failed to insert X11 Backend into event loop");

    event_loop.handle().insert_source(rx_request_fbo, move |_, _, data| {
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

    event_loop.handle().insert_source(rx_present, move |_, _, data| {
        data.common.is_next_vblank_scheduled = true;
        if let Err(err) = data.backend.x11().x11_surface.submit() {
            data.backend.x11().x11_surface.reset_buffers();
            warn!("Failed to submit buffer: {}. Retrying", err);
        };
    }).unwrap();

    event_loop.run(None, &mut state, |state: &mut State| {
        if state.common.should_stop {
            info!("Shutting down");
            state.common.loop_signal.stop();
            state.common.loop_signal.wakeup();
            return;
        }

        let _ = state.common.display_handle.flush_clients();
    }).unwrap();
}

pub struct X11Backend {
    pub x11_surface: X11Surface,
    pub mode: Mode,
}
