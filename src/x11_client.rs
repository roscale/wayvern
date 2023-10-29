use std::rc::Rc;
use std::sync::atomic::Ordering;

use log::{error, warn};
use smithay::{
    backend::{
        allocator::{
            dmabuf::DmabufAllocator,
            gbm::GbmAllocator,
            vulkan::{ImageUsageFlags, VulkanAllocator},
        },
        egl::{
            self,
        },
        vulkan::{
            self,
            version::Version,
        },
        x11::{
            self,
            X11Event,
            X11Surface,
        },
    },
    output::Mode,
    reexports::{
        ash::vk::ExtPhysicalDeviceDrmFn,
        calloop::EventLoop,
        gbm::{
            self,
            BufferObjectFlags as GbmBufferFlags,
        },
        wayland_server::Display,
    },
    utils::DeviceFd,
};
use smithay::reexports::calloop::channel;
use smithay::reexports::calloop::channel::Event;
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::wayland_server::protocol::wl_shm;

use crate::{Backend, CalloopData, flutter_engine::EmbedderChannels, ServerState};
use crate::flutter_engine::FlutterEngine;
use crate::flutter_engine::platform_channels::binary_messenger::BinaryMessenger;
use crate::flutter_engine::platform_channels::encodable_value::EncodableValue;
use crate::flutter_engine::platform_channels::method_channel::MethodChannel;
use crate::flutter_engine::platform_channels::method_result_functions::MethodResultFunctions;
use crate::flutter_engine::platform_channels::method_result_mpsc_channel::{MethodResultEnum, MethodResultMpscChannel};
use crate::flutter_engine::platform_channels::standard_method_codec::StandardMethodCodec;
use crate::input_handling::handle_input;

pub fn run_x11_client() {
    let mut event_loop = EventLoop::try_new().unwrap();
    let display: Display<ServerState<X11Data>> = Display::new().unwrap();
    let mut display_handle = display.handle();

    let x11_backend = x11::X11Backend::new().expect("Failed to initilize X11 backend");
    let x11_handle = x11_backend.handle();

    let (node, fd) = x11_handle.drm_node().expect("Could not get DRM node used by X server");

    let gbm_device = gbm::Device::new(DeviceFd::from(fd)).expect("Failed to create gbm device");
    let egl_display = egl::EGLDisplay::new(gbm_device.clone()).expect("Failed to create EGLDisplay");
    let egl_context = egl::EGLContext::new(&egl_display).expect("Failed to create EGLContext");

    let window = x11::WindowBuilder::new()
        .title("Anvil")
        .build(&x11_handle)
        .expect("Failed to create first window");

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
        },
    };

    let mut state = ServerState::new(
        display,
        event_loop.handle(),
        X11Data {
            x11_surface,
            mode: Mode {
                size: {
                    let s = window.size();
                    (s.w as i32, s.h as i32).into()
                },
                refresh: 144_000,
            },
        },
    );

    let (
        mut flutter_engine,
        EmbedderChannels {
            rx_present,
            rx_request_fbo,
            mut tx_fbo,
            tx_output_height,
            rx_baton,
        },
    ) = FlutterEngine::new(&egl_context, &state).unwrap();

    state.flutter_engine = Some(flutter_engine);

    let mut method_channel: MethodChannel = MethodChannel::new(
        state.flutter_engine_mut().binary_messenger.as_mut().unwrap(),
        "test_channel".to_string(),
        Rc::new(StandardMethodCodec::new()),
    );

    let (tx_test, rx_test) = channel::channel();

    // TODO: Provide a way to specify a channel directly, without registering a callback.
    method_channel.set_method_call_handler(Some(Box::new(move |call, result| {
        let _ = tx_test.send((call, result));
    })));

    event_loop.handle().insert_source(rx_test, move |event, _, data| {
        if let Msg((_method, mut result)) = event {
            data.state.running.store(false, Ordering::SeqCst);
            result.success(Some(EncodableValue::String("Hello from Rust!".to_string())));
        };
    }).unwrap();

    let size = window.size();
    tx_output_height.send(size.h).unwrap();
    state.flutter_engine_mut().send_window_metrics((size.w as u32, size.h as u32).into()).unwrap();

    // Mandatory formats by the Wayland spec.
    // TODO: Add more formats based on the GLES version.
    state.shm_state.update_formats([
        wl_shm::Format::Argb8888,
        wl_shm::Format::Xrgb8888,
    ]);

    let mut baton = None;

    event_loop
        .handle()
        .insert_source(x11_backend, move |event, _, data: &mut CalloopData<X11Data>| match event {
            X11Event::CloseRequested { .. } => {
                data.state.running.store(false, Ordering::SeqCst);
            }
            X11Event::Resized { new_size, .. } => {
                let size = { (new_size.w as i32, new_size.h as i32).into() };

                data.state.backend_data.mode = Mode {
                    size,
                    refresh: 144_000,
                };

                let _ = tx_output_height.send(new_size.h);
                data.state.flutter_engine().send_window_metrics((size.w as u32, size.h as u32).into()).unwrap();
            }
            X11Event::PresentCompleted { .. } | X11Event::Refresh { .. } => {
                data.state.is_next_vblank_scheduled = false;
                if let Some(baton) = data.baton.take() {
                    data.state.flutter_engine().on_vsync(baton).unwrap();
                }
            }
            X11Event::Input(event) => handle_input(&event, data),
        })
        .expect("Failed to insert X11 Backend into event loop");

    event_loop.handle().insert_source(rx_baton, move |baton, _, data| {
        if let Event::Msg(baton) = baton {
            data.baton = Some(baton);
        }
        if data.state.is_next_vblank_scheduled {
            return;
        }
        if let Some(baton) = data.baton.take() {
            data.state.flutter_engine().on_vsync(baton).unwrap();
        }

        // if let Err(err) = data.state.backend_data.x11_surface.submit() {
        //     data.state.backend_data.x11_surface.reset_buffers();
        //     warn!("Failed to submit buffer: {}. Retrying", err);
        // };
    }).unwrap();

    event_loop.handle().insert_source(rx_request_fbo, move |_, _, data| {
        match data.state.backend_data.x11_surface.buffer() {
            Ok((dmabuf, _age)) => {
                let _ = data.tx_fbo.send(Some(dmabuf));
            }
            Err(err) => {
                error!("{err}");
                let _ = data.tx_fbo.send(None);
            }
        }
    }).unwrap();

    event_loop.handle().insert_source(rx_present, move |_, _, data| {
        data.state.is_next_vblank_scheduled = true;
        if let Err(err) = data.state.backend_data.x11_surface.submit() {
            data.state.backend_data.x11_surface.reset_buffers();
            warn!("Failed to submit buffer: {}. Retrying", err);
        };
    }).unwrap();

    while state.running.load(Ordering::SeqCst) {
        let mut calloop_data = CalloopData {
            state,
            tx_fbo,
            baton,
        };

        let result = event_loop.dispatch(None, &mut calloop_data);

        CalloopData {
            state,
            tx_fbo,
            baton,
        } = calloop_data;

        if result.is_err() {
            state.running.store(false, Ordering::SeqCst);
        } else {
            display_handle.flush_clients().unwrap();
        }
    }

    // Avoid indefinite hang in the Flutter render thread waiting for new fbo.
    drop(tx_fbo);
}

pub struct X11Data {
    pub x11_surface: X11Surface,
    pub mode: Mode,
}

impl Backend for X11Data {}
