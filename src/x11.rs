use crate::flutter_engine_rs::mouse_button_tracker::{FlutterMouseButton, flutter_to_linux_button};
use crate::flutter_engine_rs::platform_channels::method_channel::MethodChannel;
use crate::flutter_engine_rs::platform_channels::standard_method_codec::StandardMethodCodec;
use crate::state::{BackendState, State};
use crate::wayland::handlers::seat::PointerFocusTarget;
use crate::wayland::handlers::xdg::MyWindowData;
use crate::x11::flutter_engine::start_flutter_engine;
use crate::{
    channel_method_call_handler::ChannelMethodCallHandler,
    flutter_engine_rs::platform_channels::encodable_value::EncodableValue,
};
use anyhow::{Context, Result};
use smithay::backend::allocator::dmabuf::AnyError;
use smithay::backend::allocator::gbm::GbmBufferFlags;
use smithay::backend::allocator::{Allocator, Slot};
use smithay::backend::input::{
    AbsolutePositionEvent, ButtonState, InputEvent, MouseButton, PointerButtonEvent,
};
use smithay::backend::x11::X11Event;
use smithay::desktop::WindowSurfaceType;
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::output::Mode;
use smithay::reexports::calloop::channel;
use smithay::reexports::drm::buffer::{DrmFourcc, DrmModifier};
use smithay::utils::{Logical, Point, SERIAL_COUNTER};
use smithay::{
    backend::{
        allocator::{
            Swapchain,
            dmabuf::{Dmabuf, DmabufAllocator},
            gbm::GbmAllocator,
        },
        drm::{DrmDeviceFd, DrmNode},
        egl::{EGLContext, EGLDevice, EGLDisplay},
        renderer::{Bind, ImportDma, gles::GlesRenderer},
        x11::{Window, WindowBuilder, X11Backend, X11Handle, X11Surface},
    },
    reexports::{calloop::LoopHandle, gbm::Device as GbmDevice, wayland_server::DisplayHandle},
    utils::DeviceFd,
};
use std::cell::RefCell;
use std::os::fd::OwnedFd;
use std::rc::Rc;
use std::time::Duration;
use tracing::error;

mod flutter_engine;

pub use flutter_engine::UserData;

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
                        if let Err(err) = state.common.flutter.engine.send_view_metrics(
                            surface.view_id,
                            size.w as usize,
                            size.h as usize,
                        ) {
                            error!(
                                "Failed to send metrics to view {}, w: {}, h: {}, {}",
                                surface.view_id, size.w, size.h, err,
                            );
                        }

                        let mode = Mode {
                            size: (size.w as i32, size.h as i32).into(),
                            refresh: 144,
                        };
                        state.common.output.set_preferred(mode);
                        state
                            .common
                            .output
                            .change_current_state(Some(mode), None, None, None);

                        break;
                    }
                }
            }
            X11Event::Input { event, .. } => match event {
                InputEvent::PointerMotionAbsolute { event } => {
                    state.backend.x11().mouse_position = (event.x(), event.y());
                    // TODO(roscale): send Add event
                    let _ = state.common.flutter.engine.motion(event.x(), event.y());
                }
                InputEvent::PointerButton { event } => {
                    let Some(button) = event.button() else {
                        return;
                    };

                    let flutter_mouse_button = match button {
                        MouseButton::Left => FlutterMouseButton::Primary,
                        MouseButton::Middle => FlutterMouseButton::Middle,
                        MouseButton::Right => FlutterMouseButton::Secondary,
                        MouseButton::Back => FlutterMouseButton::Back,
                        MouseButton::Forward => FlutterMouseButton::Forward,
                        _ => return,
                    };

                    let pressed = match event.state() {
                        ButtonState::Released => false,
                        ButtonState::Pressed => true,
                    };

                    let mouse_position = state.backend.x11().mouse_position;

                    let _ = state.common.flutter.engine.button(
                        flutter_mouse_button,
                        pressed,
                        mouse_position.0,
                        mouse_position.1,
                    );
                }
                _ => {}
            },
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

        next_view_id: 0,
        mouse_position: (0.0, 0.0),
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
            let channel::Event::Msg((method_call, mut result)) = event else {
                error!("window channel closed");
                return;
            };

            match method_call.method() {
                "setInitialWindowSize" => {
                    let arguments = method_call.arguments().unwrap();
                    let EncodableValue::List(list) = arguments else {
                        panic!();
                    };

                    let width = list[0].long_value().unwrap();
                    let height = list[1].long_value().unwrap();

                    state.common.initial_window_size = Some((width as i32, height as i32).into());

                    result.success(None);
                }
                "motion" => {
                    let arguments = method_call.arguments().unwrap();
                    let EncodableValue::List(list) = arguments else {
                        panic!();
                    };

                    let surface_id = list[0].long_value().unwrap();
                    let EncodableValue::Double(x) = list[1] else {
                        panic!();
                    };
                    let EncodableValue::Double(y) = list[2] else {
                        panic!();
                    };

                    let window = state.common.space.elements().find(|window| {
                        window
                            .user_data()
                            .get::<RefCell<MyWindowData>>()
                            .unwrap()
                            .borrow()
                            .surface_id as i64
                            == surface_id
                    });

                    let Some(window) = window else {
                        let message = format!("Window with surface id {surface_id} doesn't exist");
                        error!(message);
                        result.error("error".to_string(), message, None);
                        return;
                    };

                    let point: Point<f64, Logical> = (x, y).into();

                    if let Some((surface, location)) =
                        window.surface_under(point, WindowSurfaceType::ALL)
                    {
                        let pointer = state.common.seat.get_pointer().unwrap();
                        let location = point - location.to_f64();
                        let time = Duration::from(state.common.clock.now()).as_millis() as u32;

                        pointer.motion(
                            state,
                            Some((PointerFocusTarget::WlSurface(surface), (0.0, 0.0).into())),
                            &MotionEvent {
                                location,
                                serial: SERIAL_COUNTER.next_serial(),
                                time,
                            },
                        );
                        pointer.frame(state);
                    }

                    result.success(None);
                }
                "button" => {
                    let arguments = method_call.arguments().unwrap();
                    let EncodableValue::List(list) = arguments else {
                        panic!();
                    };

                    let surface_id = list[0].long_value().unwrap();
                    let EncodableValue::Double(x) = list[1] else {
                        panic!();
                    };
                    let EncodableValue::Double(y) = list[2] else {
                        panic!();
                    };
                    let button = list[3].long_value().unwrap() as u32;
                    let EncodableValue::Bool(pressed) = list[4] else {
                        panic!();
                    };

                    let window = state.common.space.elements().find(|window| {
                        window
                            .user_data()
                            .get::<RefCell<MyWindowData>>()
                            .unwrap()
                            .borrow()
                            .surface_id as i64
                            == surface_id
                    });

                    let Some(window) = window else {
                        let message = format!("Window with surface id {surface_id} doesn't exist");
                        error!(message);
                        result.error("error".to_string(), message, None);
                        return;
                    };

                    let point: Point<f64, Logical> = (x, y).into();

                    if let Some((surface, location)) =
                        window.surface_under(point, WindowSurfaceType::ALL)
                    {
                        let pointer = state.common.seat.get_pointer().unwrap();
                        let location = point - location.to_f64();
                        let time = Duration::from(state.common.clock.now()).as_millis() as u32;

                        pointer.button(
                            state,
                            &ButtonEvent {
                                serial: SERIAL_COUNTER.next_serial(),
                                time,
                                button: flutter_to_linux_button(button),
                                state: if pressed {
                                    ButtonState::Pressed
                                } else {
                                    ButtonState::Released
                                },
                            },
                        );
                        pointer.frame(state);
                    }

                    result.success(None);
                }

                _ => result.not_implemented(),
            }
        })
        .unwrap();
}

pub struct X11State {
    x11_handle: X11Handle,
    allocator: GbmAllocator<DrmDeviceFd>,
    pub egl_display: EGLDisplay,
    pub renderer: GlesRenderer,
    pub surfaces: Vec<Surface>,

    pub next_view_id: i64,
    pub mouse_position: (f64, f64),
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
}

impl State {
    pub fn add_x11_window(&mut self) -> Result<()> {
        let x11_state = self.backend.x11();

        let window = WindowBuilder::new()
            .title("Wayvern")
            .build(&x11_state.x11_handle)
            .with_context(|| "Failed to create window")?;

        let size = window.size();
        let fourcc = window.format();
        let modifiers = x11_state.filtered_modifiers(fourcc);

        let x11_surface = x11_state
            .x11_handle
            .create_surface(
                &window,
                x11_state.create_dmabuf_allocator(),
                modifiers.iter().copied(),
            )
            .with_context(|| "Failed to create surface")?;

        let allocator = x11_state.create_dmabuf_allocator();

        let swapchain = Swapchain::new(
            Box::new(allocator) as Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError> + 'static>,
            size.w as u32,
            size.h as u32,
            fourcc,
            modifiers,
        );

        let surface = Surface {
            window,
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

#[derive(Debug)]
pub struct Surface {
    pub window: Window,
    pub x11_surface: X11Surface,
    pub view_id: i64,
    pub swapchain: Swapchain<Box<dyn Allocator<Buffer = Dmabuf, Error = AnyError> + 'static>>,
    pub slot: Option<Slot<Dmabuf>>,
}

fn init_egl_client_side<R>(
    _dh: &DisplayHandle,
    _state: &mut State,
    _render_node: DrmNode,
    _renderer: &mut R,
) -> Result<()>
where
    R: ImportDma,
{
    // let default_feedback =
    //     DmabufFeedbackBuilder::new(render_node.dev_id(), renderer.dmabuf_formats())
    //         .build()
    //         .unwrap();
    //
    // let _dmabuf_global = state
    //     .common
    //     .dmabuf_state
    //     .create_global_with_default_feedback::<State>(dh, &default_feedback);
    //
    // info!("EGL hardware-acceleration enabled.");
    //
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
