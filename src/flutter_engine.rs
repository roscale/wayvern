use std::ffi::{c_int, CString};
use std::mem::size_of;
use std::ops::DerefMut;
use std::os::unix::ffi::OsStrExt;
use std::ptr::{null, null_mut};
use std::rc::Rc;
use std::time::Duration;
use serde_json::json;
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        egl::{
            self,
            context::{GlAttributes, GlProfile, PixelFormatRequirements},
            EGLContext,
        },
        renderer::gles::ffi::Gles2,
    },
    reexports::calloop::channel,
    utils::{Physical, Size},
};
use smithay::backend::input::{KeyState, Keycode};
use smithay::input::keyboard::ModifiersState;
use smithay::reexports::calloop::channel::Event::Msg;
use smithay::reexports::calloop::{Dispatcher, LoopHandle};
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use embedder_sys::{FlutterEngineAOTData, FlutterEngineAOTDataSource, FlutterEngineAOTDataSourceType_kFlutterEngineAOTDataSourceTypeElfPath, FlutterEngineAOTDataSource__bindgen_ty_1, FlutterEngineCreateAOTData, FlutterEngineRun, FlutterEngineSendKeyEvent, FlutterEngineUnregisterExternalTexture, FlutterKeyEvent, FlutterKeyEventType_kFlutterKeyEventTypeDown, FlutterKeyEventType_kFlutterKeyEventTypeUp};
use embedder_sys::FlutterEngineOnVsync;
use embedder_sys::FlutterEngineGetCurrentTime;
use embedder_sys::FlutterEngine as FlutterEngineHandle;
use embedder_sys::FlutterEngineSendWindowMetricsEvent;
use embedder_sys::FlutterEngineShutdown;
use embedder_sys::FlutterOpenGLRendererConfig;
use embedder_sys::FlutterProjectArgs;
use embedder_sys::FlutterRendererConfig;
use embedder_sys::FlutterRendererConfig__bindgen_ty_1;
use embedder_sys::FlutterWindowMetricsEvent;
use embedder_sys::FLUTTER_ENGINE_VERSION;
use crate::flutter_engine::callbacks::{
    clear_current,
    fbo_callback,
    make_current,
    make_resource_current,
    present_with_info,
    surface_transformation,
};
use crate::flutter_engine::callbacks::{gl_external_texture_frame_callback, platform_message_callback, populate_existing_damage, post_task_callback, runs_task_on_current_thread_callback, vsync_callback};
use embedder_sys::{FlutterCustomTaskRunners, FlutterEngineMarkExternalTextureFrameAvailable, FlutterEngineRegisterExternalTexture, FlutterEngineRunTask, FlutterEngineSendPointerEvent, FlutterPointerEvent, FlutterTaskRunnerDescription};
use platform_channels::basic_message_channel::BasicMessageChannel;
use platform_channels::binary_messenger::BinaryReply;
use platform_channels::binary_messenger_impl::BinaryMessengerImpl;
use platform_channels::json_message_codec::JsonMessageCodec;
use platform_channels::message_codec::MessageCodec;
use platform_channels::method_channel::MethodChannel;
use platform_channels::method_codec::MethodCodec;
use platform_channels::method_result::MethodResult;
use crate::flutter_engine::task_runner::TaskRunner;
use crate::flutter_engine::text_input::TextInput;
use crate::gles_framebuffer_importer::GlesFramebufferImporter;
use crate::keyboard::glfw_key_codes::{get_glfw_keycode, get_glfw_modifiers};
use crate::mouse_button_tracker::MouseButtonTracker;
use crate::state::State;

mod callbacks;
pub mod task_runner;
pub mod wayland_messages;
pub mod method_result_mpsc_channel;
mod text_input;

/// Wrap the handle for various safety reasons:
/// - Clone & Copy is willingly not implemented to avoid using the engine after being shut down.
/// - Send is not implemented because all its methods must be called from the thread the engine was created.
pub struct FlutterEngine {
    pub handle: FlutterEngineHandle,
    data: FlutterEngineData,
    pub task_runner: TaskRunner,
    current_thread_id: std::thread::ThreadId,
    pub(crate) mouse_button_tracker: MouseButtonTracker,
    pub binary_messenger: Option<BinaryMessengerImpl>,
    pub text_input: TextInput,
}

/// I don't want people to clone it because it's UB to call [FlutterEngine::on_vsync] multiple times
/// with the same baton, which will most probably segfault.
pub struct Baton(isize);

impl FlutterEngine {
    pub fn new(root_egl_context: &EGLContext, loop_handle: &LoopHandle<State>) -> Result<(Box<Self>, EmbedderChannels), Box<dyn std::error::Error>> {
        let (tx_present, rx_present) = channel::channel::<()>();
        let (tx_request_fbo, rx_request_fbo) = channel::channel::<()>();
        let (tx_fbo, rx_fbo) = channel::channel::<Option<Dmabuf>>();
        let (tx_baton, rx_baton) = channel::channel::<Baton>();
        let (tx_reschedule_task_runner_timer, rx_reschedule_task_runner_timer) = channel::channel::<Duration>();
        let (tx_request_external_texture_name, rx_request_external_texture_name) = channel::channel::<i64>();
        let (tx_external_texture_name, rx_external_texture_name) = channel::channel::<(u32, u32)>();

        let flutter_engine_channels = FlutterEngineChannels {
            tx_present,
            tx_request_fbo,
            rx_fbo,
            tx_baton,
            tx_request_external_texture_name,
            rx_external_texture_name,
        };

        let embedder_channels = EmbedderChannels {
            rx_present,
            rx_request_fbo,
            tx_fbo,
            rx_baton,
            rx_request_external_texture_name,
            tx_external_texture_name,
        };

        let flutter_engine_build = option_env!("FLUTTER_ENGINE").unwrap_or("debug");

        let executable_path = std::fs::canonicalize("/proc/self/exe")?;

        let bundle_root = if option_env!("BUNDLE").is_some() {
            "".to_string()
        } else {
            format!("wayvern_flutter/build/linux/x64/{flutter_engine_build}/bundle/")
        };

        let assets_path = CString::new(format!("{bundle_root}data/flutter_assets"))?;
        let icu_data_path = CString::new(format!("{bundle_root}data/icudtl.dat"))?;
        let executable_path = CString::new(executable_path.as_os_str().as_bytes())?;
        let observatory_port = CString::new("--observatory-port=12345")?;
        let disable_service_auth_codes = CString::new("--disable-service-auth-codes")?;

        let command_line_argv = [
            executable_path.as_ptr(),
            observatory_port.as_ptr(),
            disable_service_auth_codes.as_ptr(),
        ];

        let elf_path = CString::new(format!("{bundle_root}lib/libapp.so"))?;
        let mut aot_data: FlutterEngineAOTData = null_mut();

        if flutter_engine_build != "debug" {
            let aot_data_source = FlutterEngineAOTDataSource {
                type_: FlutterEngineAOTDataSourceType_kFlutterEngineAOTDataSourceTypeElfPath,
                __bindgen_anon_1: FlutterEngineAOTDataSource__bindgen_ty_1 {
                    elf_path: elf_path.as_ptr(),
                },
            };
            let result = unsafe {
                FlutterEngineCreateAOTData(&aot_data_source as *const _, &mut aot_data as *mut _)
            };
            if result != 0 {
                return Err(format!("Could not create AOT data, error {result}").into());
            }
        }
        
        let mut this = Box::new(Self {
            handle: null_mut(),
            data: FlutterEngineData::new(root_egl_context, flutter_engine_channels)?,
            task_runner: TaskRunner::new(tx_reschedule_task_runner_timer),
            current_thread_id: std::thread::current().id(),
            mouse_button_tracker: MouseButtonTracker::new(),
            binary_messenger: None,
            text_input: TextInput::new(),
        });

        let task_runner_description = FlutterTaskRunnerDescription {
            struct_size: size_of::<FlutterTaskRunnerDescription>(),
            user_data: this.deref_mut() as *const _ as *mut _,
            runs_task_on_current_thread_callback: Some(runs_task_on_current_thread_callback),
            post_task_callback: Some(post_task_callback),
            identifier: 1,
        };

        let task_runners = FlutterCustomTaskRunners {
            struct_size: size_of::<FlutterCustomTaskRunners>(),
            platform_task_runner: &task_runner_description,
            render_task_runner: null(),
            thread_priority_setter: None,
        };

        let project_args = FlutterProjectArgs {
            struct_size: size_of::<FlutterProjectArgs>(),
            assets_path: assets_path.as_ptr(),
            main_path__unused__: null(),
            packages_path__unused__: null(),
            icu_data_path: icu_data_path.as_ptr(),
            command_line_argc: command_line_argv.len() as c_int,
            command_line_argv: command_line_argv.as_ptr(),
            platform_message_callback: Some(platform_message_callback),
            vm_snapshot_data: null(),
            vm_snapshot_data_size: 0,
            vm_snapshot_instructions: null(),
            vm_snapshot_instructions_size: 0,
            isolate_snapshot_data: null(),
            isolate_snapshot_data_size: 0,
            isolate_snapshot_instructions: null(),
            isolate_snapshot_instructions_size: 0,
            root_isolate_create_callback: None,
            update_semantics_node_callback: None,
            update_semantics_custom_action_callback: None,
            persistent_cache_path: null(),
            is_persistent_cache_read_only: false,
            vsync_callback: Some(vsync_callback),
            custom_dart_entrypoint: null(),
            custom_task_runners: &task_runners,
            shutdown_dart_vm_when_done: true,
            compositor: null(),
            dart_old_gen_heap_size: 0,
            aot_data,
            compute_platform_resolved_locale_callback: None,
            dart_entrypoint_argc: 0,
            dart_entrypoint_argv: null(),
            log_message_callback: None,
            log_tag: null(),
            on_pre_engine_restart_callback: None,
            update_semantics_callback: None,
            update_semantics_callback2: None,
            channel_update_callback: None,
        };

        let renderer_config = FlutterRendererConfig {
            type_: 0,
            __bindgen_anon_1: FlutterRendererConfig__bindgen_ty_1 {
                open_gl: FlutterOpenGLRendererConfig {
                    struct_size: size_of::<FlutterOpenGLRendererConfig>(),
                    make_current: Some(make_current),
                    clear_current: Some(clear_current),
                    present: None,
                    fbo_callback: Some(fbo_callback),
                    make_resource_current: Some(make_resource_current),
                    // Flutter must request another framebuffer every frame
                    // because we're using a triple-buffered swapchain.
                    fbo_reset_after_present: true,
                    surface_transformation: Some(surface_transformation),
                    gl_proc_resolver: None,
                    gl_external_texture_frame_callback: Some(gl_external_texture_frame_callback),
                    fbo_with_frame_info_callback: None,
                    present_with_info: Some(present_with_info),
                    populate_existing_damage: Some(populate_existing_damage),
                }
            },
        };

        let mut flutter_engine: FlutterEngineHandle = null_mut();
        let result = unsafe {
            FlutterEngineRun(
                FLUTTER_ENGINE_VERSION as usize,
                &renderer_config,
                &project_args,
                this.deref_mut() as *const _ as *mut _,
                &mut flutter_engine,
            )
        };

        if result != 0 {
            return Err(format!("Could not initalize the Flutter engine, error {result}").into());
        }

        this.handle = flutter_engine;

        let task_runner_timer_dispatcher = Dispatcher::new(Timer::immediate(), move |_deadline, _, data: &mut State| {
            let duration = data.common.flutter_engine.task_runner.execute_expired_tasks(move |task| {
                unsafe { FlutterEngineRunTask(flutter_engine, task as *const _) };
            });
            TimeoutAction::ToDuration(duration)
        });

        let task_runner_timer_registration_token = loop_handle.register_dispatcher(task_runner_timer_dispatcher.clone()).unwrap();

        loop_handle.insert_source(rx_reschedule_task_runner_timer, move |event, _, data: &mut State| {
            if let Msg(duration) = event {
                task_runner_timer_dispatcher.as_source_mut().set_duration(duration);
                data.common.loop_handle.update(&task_runner_timer_registration_token).unwrap();
            }
        }).unwrap();

        this.binary_messenger = Some(BinaryMessengerImpl::new(flutter_engine));

        Ok((this, embedder_channels))
    }

    pub fn current_time_ns() -> u64 {
        unsafe { FlutterEngineGetCurrentTime() }
    }

    pub fn current_time_us() -> u64 {
        unsafe { FlutterEngineGetCurrentTime() / 1000 }
    }

    pub fn set_canvas_size(&mut self, size: Size<u32, Physical>, y_flipped: bool) -> Result<(), Box<dyn std::error::Error>> {
        if y_flipped {
            self.data.canvas_height = Some(size.h);
        }

        let event = FlutterWindowMetricsEvent {
            struct_size: size_of::<FlutterWindowMetricsEvent>(),
            width: size.w as usize,
            height: size.h as usize,
            pixel_ratio: 1.0,
            left: 0,
            top: 0,
            physical_view_inset_top: 0.0,
            physical_view_inset_right: 0.0,
            physical_view_inset_bottom: 0.0,
            physical_view_inset_left: 0.0,
            display_id: 0,
        };

        let result = unsafe { FlutterEngineSendWindowMetricsEvent(self.handle, &event as *const _) };
        if result != 0 {
            return Err(format!("Could not send window metrics event, error {result}").into());
        }
        Ok(())
    }

    pub fn on_vsync(&self, baton: Baton) -> Result<(), Box<dyn std::error::Error>> {
        let now = unsafe { FlutterEngineGetCurrentTime() };
        let next_frame = now + (1_000_000_000 / 144);
        let result = unsafe { FlutterEngineOnVsync(self.handle, baton.0, now, next_frame) };
        if result != 0 {
            return Err(format!("Could not send vsync baton, error {result}").into());
        }
        Ok(())
    }

    pub fn send_pointer_event(&self, event: FlutterPointerEvent) -> Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { FlutterEngineSendPointerEvent(self.handle, &event as *const _, 1) };
        if result != 0 {
            return Err(format!("Could not send pointer event, error {result}").into());
        }
        Ok(())
    }

    pub fn send_key_event(&mut self, key_event: KeyEvent, handled_channel: channel::Sender<(KeyEvent, bool)>) -> Result<(), Box<dyn std::error::Error>> {
        let codec = JsonMessageCodec::new();

        let physical = key_event.key_code.raw() + 8;
        let logical = get_glfw_keycode(key_event.key_code.raw());

        let character = key_event.codepoint.map(|c| CString::new(c.to_string()).unwrap());

        // https://api.flutter.dev/flutter/services/KeyEventManager-class.html
        unsafe {
            FlutterEngineSendKeyEvent(
                self.handle,
                &FlutterKeyEvent {
                    struct_size: size_of::<FlutterKeyEvent>(),
                    timestamp: FlutterEngine::current_time_us() as f64,
                    type_: if key_event.state == KeyState::Pressed {
                        FlutterKeyEventType_kFlutterKeyEventTypeDown
                    } else {
                        FlutterKeyEventType_kFlutterKeyEventTypeUp
                    },
                    physical: physical as u64,
                    logical: logical as u64,
                    character: character.map(|c| c.as_ptr()).unwrap_or(null()),
                    synthesized: false,
                } as *const _,
                None,
                null_mut(),
            );
        };

        self.send_message(
            codec,
            "flutter/keyevent",
            json!({
                "keymap": "linux",
                "toolkit": "glfw",
                "keyCode": logical,
                "specifiedLogicalKey": key_event.specified_logical_key,
                "scanCode": physical,
                "modifiers": get_glfw_modifiers(key_event.mods),
                "unicodeScalarValues": key_event.codepoint.map(|c| c as u32),
                "type": if key_event.state == KeyState::Pressed { "keydown" } else { "keyup" },
            }),
            Some(Box::new(move |response: Option<&[u8]>| {
                // This is the callback that will be called when Flutter replies.
                // Flutter always replies with a single `handled` boolean.
                // If its value is true, some widget listening to keyboard shortcuts probably handled this event.

                let response = match response {
                    Some(response) => response,
                    None => return,
                };

                let message = codec.decode_message(response).unwrap();
                let handled = message["handled"].as_bool().unwrap();

                // We would normally call `keyboard.input_forward` here to forward the event to a Wayland client,
                // but we need `data` and we can't just capture it by reference.
                // Send key event info and Flutter's response over an MPSC channel.
                // The receiver `rx_flutter_handled_key_event` is registered to the event loop with a callback
                // that will continue processing the event.
                // This callback is defined in the constructor of `ServerState`.
                handled_channel.send((key_event, handled)).unwrap();
            })),
        );
        Ok(())
    }

    pub fn register_external_texture(&self, texture_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { FlutterEngineRegisterExternalTexture(self.handle, texture_id) };
        if result != 0 {
            return Err(format!("Could not register external texture, error {result}").into());
        }
        Ok(())
    }

    pub fn unregister_external_texture(&self, texture_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { FlutterEngineUnregisterExternalTexture(self.handle, texture_id) };
        if result != 0 {
            return Err(format!("Could not unregister external texture, error {result}").into());
        }
        Ok(())
    }

    pub fn mark_external_texture_frame_available(&self, texture_id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { FlutterEngineMarkExternalTextureFrameAvailable(self.handle, texture_id) };
        if result != 0 {
            return Err(format!("Could not mark external texture frame available, error {result}").into());
        }
        Ok(())
    }

    pub fn invoke_method<T: 'static, C: MethodCodec<T> + 'static>(
        &mut self,
        codec: C,
        channel_name: &str,
        method: &str,
        arguments: Option<Box<T>>,
        result: Option<Box<dyn MethodResult<T>>>,
    ) {
        let codec = Rc::new(codec);
        let mut method_channel = MethodChannel::new(
            self.binary_messenger.as_mut().unwrap(),
            channel_name.to_string(),
            codec,
        );
        method_channel.invoke_method(method, arguments, result);
    }

    pub fn send_message<T: 'static, C: MessageCodec<T> + 'static>(
        &mut self,
        codec: C,
        channel_name: &str,
        message: T,
        reply: BinaryReply,
    ) {
        let codec = Rc::new(codec);
        let mut basic_message_channel = BasicMessageChannel::new(
            self.binary_messenger.as_mut().unwrap(),
            channel_name.to_string(),
            codec,
        );
        basic_message_channel.send(&message, reply);
    }
}

impl Drop for FlutterEngine {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let _ = unsafe { FlutterEngineShutdown(self.handle) };
        }
    }
}

struct FlutterEngineData {
    gl: Gles2,
    main_egl_context: EGLContext,
    resource_egl_context: EGLContext,
    canvas_height: Option<u32>,
    channels: FlutterEngineChannels,
    framebuffer_importer: GlesFramebufferImporter,
}

// Ironically, EGLContext which contains EGLDisplay is Send, but EGLDisplay is not.
// This impl is not needed, but it's here to make it explicit that it's safe to send this struct
// to the Flutter render thread.
unsafe impl Send for FlutterEngineData {}

impl FlutterEngineData {
    fn new(root_egl_context: &EGLContext, channels: FlutterEngineChannels) -> Result<Self, Box<dyn std::error::Error>> {
        unsafe { root_egl_context.make_current()? };

        let egl_display = root_egl_context.display();
        let gl_attributes = GlAttributes {
            version: (2, 0),
            profile: Some(GlProfile::Core),
            debug: false,
            vsync: false,
        };
        let pixel_format_requirements = PixelFormatRequirements::_8_bit();

        Ok(Self {
            gl: Gles2::load_with(|s| unsafe { egl::get_proc_address(s) } as *const _),
            main_egl_context: EGLContext::new_shared_with_config(&egl_display, &root_egl_context, gl_attributes, pixel_format_requirements)?,
            resource_egl_context: EGLContext::new_shared_with_config(&egl_display, &root_egl_context, gl_attributes, pixel_format_requirements)?,
            canvas_height: None,
            channels,
            framebuffer_importer: unsafe { GlesFramebufferImporter::new(egl_display.clone())? },
        })
    }
}

pub struct FlutterEngineChannels {
    tx_present: channel::Sender<()>,
    tx_request_fbo: channel::Sender<()>,
    rx_fbo: channel::Channel<Option<Dmabuf>>,
    tx_baton: channel::Sender<Baton>,
    tx_request_external_texture_name: channel::Sender<i64>,
    rx_external_texture_name: channel::Channel<(u32, u32)>,
}

pub struct EmbedderChannels {
    pub rx_present: channel::Channel<()>,
    pub rx_request_fbo: channel::Channel<()>,
    pub tx_fbo: channel::Sender<Option<Dmabuf>>,
    pub rx_baton: channel::Channel<Baton>,
    pub rx_request_external_texture_name: channel::Channel<i64>,
    pub tx_external_texture_name: channel::Sender<(u32, u32)>,
}

#[derive(Copy, Clone)]
pub struct KeyEvent {
    pub key_code: Keycode,
    pub specified_logical_key: Option<u32>,
    pub codepoint: Option<char>,
    pub state: KeyState,
    pub time: u32,
    pub mods: ModifiersState,
    pub mods_changed: bool,
}
