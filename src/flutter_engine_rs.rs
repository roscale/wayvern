mod binary_messenger_impl;
mod engine_callbacks;
mod engine_handle;
mod ffi;
mod flutter_engine_sys;
pub mod keyboard;
mod monotonic_time;
pub mod mouse_button_tracker;
pub mod platform_channels;
mod task_runner;
mod user_data;

use crate::flutter_engine_rs::{
    binary_messenger_impl::BinaryMessengerImpl,
    ffi::key_event_callback,
    keyboard::{
        gtk_to_logical::GTK_TO_LOGICAL_MAP,
        modifiers::{FlutterModifiers, get_modifiers_bitfield},
        xkb_to_physical::XKB_TO_PHYSICAL_MAP,
    },
    monotonic_time::monotonic_time,
    mouse_button_tracker::{FlutterMouseButton, FlutterMouseButtonTracker},
    platform_channels::{
        basic_message_channel::BasicMessageChannel, json_message_codec::JsonMessageCodec,
    },
};
use anyhow::{Result, ensure};
pub use engine_callbacks::EngineCallbacks;
use engine_handle::EngineHandle;
use flutter_engine_sys::*;
use serde_json::json;
use smithay::reexports::winit::keyboard::Key;
use std::{
    ffi::{CString, c_int, c_void},
    mem::size_of,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf, absolute},
    ptr::{null, null_mut},
    rc::Rc,
    sync::mpsc,
    time::Instant,
};
use tracing::error;
use user_data::EngineUserData;

pub struct Engine<UserData> {
    handle: EngineHandle,
    engine_user_data: Box<EngineUserData<UserData>>,
    platform_task_runner: task_runner::TaskRunner,
    mouse_button_tracker: FlutterMouseButtonTracker,
    mouse_position: (f64, f64),
}

impl<UserData> Engine<UserData> {
    pub fn new(
        flutter_project: &Path,
        aot_mode: bool,
        callbacks: EngineCallbacks<UserData>,
        user_data: UserData,
    ) -> Result<Self> {
        let EngineCallbacks {
            proc_resolver,
            external_texture_frame,
            make_current,
            clear_current,
            make_resource_current,
            request_texture,
            vsync,
            present,
        } = callbacks;

        let bundle_path = get_bundle_path(flutter_project, aot_mode);

        let assets_path = CString::new(
            bundle_path
                .join("data/flutter_assets")
                .as_os_str()
                .as_bytes(),
        )?;

        let icu_data_path =
            CString::new(bundle_path.join("data/icudtl.dat").as_os_str().as_bytes())?;

        let executable_path = CString::new(
            std::fs::canonicalize("/proc/self/exe")?
                .as_os_str()
                .as_bytes(),
        )?;

        let observatory_port = CString::new("--observatory-port=12345")?;
        let disable_service_auth_codes = CString::new("--disable-service-auth-codes")?;

        let command_line_argv = [
            executable_path.as_ptr(),
            observatory_port.as_ptr(),
            disable_service_auth_codes.as_ptr(),
        ];

        let elf_path = CString::new(bundle_path.join("lib/libapp.so").as_os_str().as_bytes())?;

        let platform_task_runner = task_runner::spawn_task_runner();
        let binary_messenger = BinaryMessengerImpl::new((*platform_task_runner.poster).clone());

        let mut engine_user_data = Box::new(user_data::EngineUserData {
            proc_resolver,
            external_texture_frame,
            vsync,
            make_current,
            clear_current,
            make_resource_current,
            present,
            request_texture,
            binary_messenger,
            user_data,
        });

        let mut handle: FlutterEngine = null_mut();

        let result = unsafe {
            FlutterEngineInitialize(
                FLUTTER_ENGINE_VERSION as usize,
                &FlutterRendererConfig {
                    type_: FlutterRendererType_kOpenGL,
                    __bindgen_anon_1: FlutterRendererConfig__bindgen_ty_1 {
                        open_gl: FlutterOpenGLRendererConfig {
                            struct_size: size_of::<FlutterOpenGLRendererConfig>(),
                            make_current: Some(ffi::make_current_callback::<UserData>),
                            clear_current: Some(ffi::clear_current_callback::<UserData>),
                            present: Some(ffi::present),
                            fbo_callback: Some(ffi::fbo_callback),
                            make_resource_current: Some(
                                ffi::make_resource_current_callback::<UserData>,
                            ),
                            fbo_reset_after_present: true,
                            surface_transformation: None,
                            gl_proc_resolver: Some(ffi::gl_proc_resolver_callback::<UserData>),
                            gl_external_texture_frame_callback: Some(
                                ffi::gl_external_texture_frame_callback::<UserData>,
                            ),
                            fbo_with_frame_info_callback: None,
                            present_with_info: None,
                            populate_existing_damage: None,
                        },
                    },
                },
                &FlutterProjectArgs {
                    struct_size: size_of::<FlutterProjectArgs>(),
                    assets_path: assets_path.as_ptr(),
                    main_path__unused__: null(),
                    packages_path__unused__: null(),
                    icu_data_path: icu_data_path.as_ptr(),
                    command_line_argc: command_line_argv.len() as c_int,
                    command_line_argv: command_line_argv.as_ptr(),
                    platform_message_callback: Some(ffi::platform_message_callback::<UserData>),
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
                    vsync_callback: Some(ffi::vsync_callback::<UserData>),
                    custom_dart_entrypoint: null(),
                    custom_task_runners: &FlutterCustomTaskRunners {
                        struct_size: size_of::<FlutterCustomTaskRunners>(),
                        platform_task_runner: &FlutterTaskRunnerDescription {
                            struct_size: size_of::<FlutterTaskRunnerDescription>(),
                            user_data: &*platform_task_runner.poster as *const _ as *mut _,
                            runs_task_on_current_thread_callback: Some(
                                ffi::runs_task_on_current_thread_callback,
                            ),
                            post_task_callback: Some(ffi::post_task_callback),
                            identifier: 0,
                            destruction_callback: None,
                        },
                        render_task_runner: null_mut(),
                        thread_priority_setter: None,
                        ui_task_runner: null_mut(),
                    },
                    shutdown_dart_vm_when_done: true,
                    compositor: &FlutterCompositor {
                        struct_size: std::mem::size_of::<FlutterCompositor>(),
                        user_data: &*engine_user_data as *const _ as *mut _,
                        create_backing_store_callback: Some(
                            ffi::create_backing_store_callback::<UserData>,
                        ),
                        collect_backing_store_callback: Some(ffi::collect_backing_store_callback),
                        present_layers_callback: None,
                        present_view_callback: Some(ffi::present_view_callback::<UserData>),
                        avoid_backing_store_cache: true,
                    },
                    dart_old_gen_heap_size: 0,
                    aot_data: {
                        let mut aot_data: FlutterEngineAOTData = null_mut();

                        if aot_mode {
                            let result = FlutterEngineCreateAOTData(
                                &FlutterEngineAOTDataSource {
                                    type_: FlutterEngineAOTDataSourceType_kFlutterEngineAOTDataSourceTypeElfPath,
                                    __bindgen_anon_1: FlutterEngineAOTDataSource__bindgen_ty_1 {
                                        elf_path: elf_path.as_ptr(),
                                    },
                                } as *const _,
                                &mut aot_data as *mut _,
                            );

                            ensure!(result == 0, "Could not create AOT data, error {result}");
                        }

                        aot_data
                    },
                    compute_platform_resolved_locale_callback: None,
                    dart_entrypoint_argc: 0,
                    dart_entrypoint_argv: null(),
                    log_message_callback: Some(ffi::log_message_callback),
                    log_tag: null(),
                    on_pre_engine_restart_callback: None,
                    update_semantics_callback: None,
                    update_semantics_callback2: None,
                    channel_update_callback: None,
                    view_focus_change_request_callback: None,
                    engine_id: 0,
                },
                // SAFETY: The value is boxed, so its address stays the same even after being moved into Self.
                &*engine_user_data as *const _ as *mut _,
                &mut handle,
            )
        };

        engine_user_data.binary_messenger.set_flutter_engine(handle);

        ensure!(
            result == 0,
            "Could not initalize the Flutter engine, error {result}"
        );

        platform_task_runner
            .engine_handle_tx
            .send(EngineHandle(handle))
            .expect("Failed to send Flutter engine handle to task runner");

        let result = platform_task_runner
            .poster
            .post_callback(move || unsafe { FlutterEngineRunInitialized(handle) });

        ensure!(
            result == 0,
            "Could not run the Flutter engine, error {result}"
        );

        let flutter_engine = Self {
            handle: EngineHandle(handle),
            engine_user_data,
            platform_task_runner,
            mouse_button_tracker: FlutterMouseButtonTracker::new(),
            mouse_position: (0.0, 0.0),
        };

        Ok(flutter_engine)
    }

    pub fn add_view(
        &mut self,
        view_id: i64,
        width: usize,
        height: usize,
        callback: impl Fn(bool),
    ) -> Result<()> {
        // View 0 always exists, but it doesn't function until metrics have been set.
        if view_id == 0 {
            self.send_view_metrics(view_id, width, height)?;
            callback(true);
            return Ok(());
        }

        let view_metrics = create_view_metrics(width, height, view_id);

        // Double boxing because *dyn Fn(bool) is a fat pointer.
        let callback: *mut Box<dyn Fn(bool)> = Box::into_raw(Box::new(Box::new(callback)));

        let info = FlutterAddViewInfo {
            struct_size: size_of::<FlutterAddViewInfo>(),
            view_id,
            view_metrics: &view_metrics as *const _,
            user_data: callback as *mut c_void,
            add_view_callback: Some(ffi::add_view_callback),
        };

        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe { FlutterEngineAddView(*handle, &info) });

        ensure!(result == 0, "Could add view {view_id}, error {result}");

        Ok(())
    }

    pub fn send_view_metrics(&mut self, view_id: i64, width: usize, height: usize) -> Result<()> {
        let view_metrics = create_view_metrics(width, height, view_id);

        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe {
                FlutterEngineSendWindowMetricsEvent(*handle, &view_metrics as *const _)
            });

        ensure!(
            result == 0,
            "Could not send window metrics event, error {result}"
        );

        Ok(())
    }

    pub fn current_time_ns(&self) -> u64 {
        monotonic_time()
    }

    pub fn current_time_us(&self) -> u64 {
        monotonic_time() / 1000
    }

    pub fn current_time_us_f64(&self) -> f64 {
        monotonic_time() as f64 / 1000.0
    }

    pub fn on_vsync(
        &self,
        baton: isize,
        frame_start_time_nanos: u64,
        frame_target_time_nanos: u64,
    ) -> Result<()> {
        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe {
                FlutterEngineOnVsync(
                    *handle,
                    baton,
                    frame_start_time_nanos,
                    frame_target_time_nanos,
                )
            });

        ensure!(result == 0, "Could not send vsync baton, error {result}");

        Ok(())
    }

    pub fn binary_messenger(&mut self) -> &mut BinaryMessengerImpl {
        &mut self.engine_user_data.binary_messenger
    }

    pub fn register_external_texture(&self, texture_id: i64) -> Result<()> {
        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe {
                FlutterEngineRegisterExternalTexture(*handle, texture_id)
            });

        ensure!(
            result == 0,
            "Could not register external texture, error {result}"
        );

        Ok(())
    }

    pub fn _unregister_external_texture(&self, texture_id: i64) -> Result<()> {
        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe {
                FlutterEngineUnregisterExternalTexture(*handle, texture_id)
            });

        ensure!(
            result == 0,
            "Could not unregister external texture, error {result}"
        );

        Ok(())
    }

    pub fn mark_external_texture_frame_available(&self, texture_id: i64) -> Result<()> {
        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe {
                FlutterEngineMarkExternalTextureFrameAvailable(*handle, texture_id)
            });

        ensure!(
            result == 0,
            "Could not mark external texture frame available, error {result}"
        );

        Ok(())
    }

    pub fn motion(&mut self, x: f64, y: f64) -> Result<()> {
        self.mouse_position = (x, y);

        let event = FlutterPointerEvent {
            struct_size: size_of::<FlutterPointerEvent>(),
            phase: if self.mouse_button_tracker.are_any_buttons_pressed() {
                FlutterPointerPhase_kMove
            } else {
                FlutterPointerPhase_kHover
            },
            timestamp: self.current_time_us() as usize,
            x,
            y,
            device: 0,
            signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
            scroll_delta_x: 0.0,
            scroll_delta_y: 0.0,
            device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
            buttons: self.mouse_button_tracker.get_bitmask(),
            pan_x: 0.0,
            pan_y: 0.0,
            scale: 0.0,
            rotation: 0.0,
            view_id: 0,
        };

        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe { FlutterEngineSendPointerEvent(*handle, &event, 1) });

        ensure!(
            result == 0,
            "Could not send motion pointer event, error {result}"
        );

        Ok(())
    }

    pub fn button(&mut self, button: FlutterMouseButton, pressed: bool) -> Result<()> {
        let phase = if pressed {
            let are_any_buttons_pressed = self.mouse_button_tracker.are_any_buttons_pressed();
            self.mouse_button_tracker.press(button);

            if are_any_buttons_pressed {
                FlutterPointerPhase_kMove
            } else {
                FlutterPointerPhase_kDown
            }
        } else {
            self.mouse_button_tracker.release(button);
            let are_any_buttons_pressed = self.mouse_button_tracker.are_any_buttons_pressed();

            if are_any_buttons_pressed {
                FlutterPointerPhase_kMove
            } else {
                FlutterPointerPhase_kUp
            }
        };

        let event = FlutterPointerEvent {
            struct_size: size_of::<FlutterPointerEvent>(),
            phase,
            timestamp: self.current_time_us() as usize,
            x: self.mouse_position.0,
            y: self.mouse_position.1,
            device: 0,
            signal_kind: FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
            scroll_delta_x: 0.0,
            scroll_delta_y: 0.0,
            device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
            buttons: self.mouse_button_tracker.get_bitmask(),
            pan_x: 0.0,
            pan_y: 0.0,
            scale: 0.0,
            rotation: 0.0,
            view_id: 0,
        };

        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe { FlutterEngineSendPointerEvent(*handle, &event, 1) });

        ensure!(
            result == 0,
            "Could not send button pointer event, error {result}"
        );

        Ok(())
    }

    pub fn axis(&mut self, dx: f64, dy: f64) -> Result<()> {
        let phase = if self.mouse_button_tracker.are_any_buttons_pressed() {
            FlutterPointerPhase_kMove
        } else {
            FlutterPointerPhase_kDown
        };

        let event = FlutterPointerEvent {
            struct_size: size_of::<FlutterPointerEvent>(),
            phase,
            timestamp: self.current_time_us() as usize,
            x: self.mouse_position.0,
            y: self.mouse_position.1,
            device: 0,
            signal_kind: if (dx, dy) != (0.0, 0.0) {
                FlutterPointerSignalKind_kFlutterPointerSignalKindScroll
            } else {
                FlutterPointerSignalKind_kFlutterPointerSignalKindScrollInertiaCancel
            },
            scroll_delta_x: dx,
            scroll_delta_y: dy,
            device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
            buttons: self.mouse_button_tracker.get_bitmask(),
            pan_x: 0.0,
            pan_y: 0.0,
            scale: 0.0,
            rotation: 0.0,
            view_id: 0,
        };

        let handle = self.handle;

        let result = self
            .platform_task_runner
            .poster
            .post_callback(move || unsafe { FlutterEngineSendPointerEvent(*handle, &event, 1) });

        ensure!(
            result == 0,
            "Could not send button pointer event, error {result}"
        );

        Ok(())
    }

    pub fn send_key(
        &mut self,
        key_state: KeyState,
        keycode: u32,
        keysym: u32,
        character: Option<char>,
        modifiers: FlutterModifiers,
        synthesized: bool,
    ) -> Result<bool> {
        // https://github.com/flutter/flutter/blob/3bf280c86fb9c083e05a7cd60f27f8a5e5c21424/engine/src/flutter/shell/platform/linux/fl_key_channel_responder.cc

        let (mut key_handled_tx, key_handled_rx) = mpsc::channel::<bool>();

        let result = {
            let physical = match XKB_TO_PHYSICAL_MAP.get(&keycode).copied() {
                Some(physical) => physical,
                None => keycode as u64,
            };

            let logical = match GTK_TO_LOGICAL_MAP.get(&keysym).copied() {
                Some(logical) => logical,
                None => keysym as u64,
            };

            let character = character.and_then(|c| match key_state {
                KeyState::Down | KeyState::Repeat => CString::new(c.to_string()).ok(),
                KeyState::Up => None,
            });

            let event = {
                FlutterKeyEvent {
                    struct_size: size_of::<FlutterKeyEvent>(),
                    timestamp: self.current_time_us_f64(),
                    type_: match key_state {
                        KeyState::Down => FlutterKeyEventType_kFlutterKeyEventTypeDown,
                        KeyState::Up => FlutterKeyEventType_kFlutterKeyEventTypeUp,
                        KeyState::Repeat => FlutterKeyEventType_kFlutterKeyEventTypeRepeat,
                    },
                    physical,
                    logical,
                    character: character.as_ref().map(|c| c.as_ptr()).unwrap_or(null()),
                    synthesized,
                    device_type: FlutterKeyEventDeviceType_kFlutterKeyEventDeviceTypeKeyboard,
                }
            };

            let handle = self.handle;

            self.platform_task_runner
                .poster
                .post_callback(move || unsafe {
                    FlutterEngineSendKeyEvent(
                        *handle,
                        &event,
                        Some(key_event_callback),
                        Box::into_raw(Box::new(key_handled_tx)) as *mut c_void,
                    )
                })
        };

        ensure!(result == 0, "Failed to send key event, error {result}");

        // https://github.com/flutter/flutter/blob/90a892293c166a96acca5743876a96fbc244be8c/packages/flutter/lib/src/services/hardware_keyboard.dart#L895

        BasicMessageChannel::new(
            self.binary_messenger(),
            "flutter/keyevent".to_string(),
            Rc::new(JsonMessageCodec::new()),
        )
        .send(
            &json!({
                "keymap": "linux",
                "toolkit": "gtk",
                "scanCode": keycode,
                "keyCode": keysym,
                "unicodeScalarValues": character.map(|c| c as u32).unwrap_or(0),
                "modifiers": get_modifiers_bitfield(modifiers),
                "type": match key_state {
                    KeyState::Down | KeyState::Repeat => "keydown",
                    KeyState::Up => "keyup",
                },
            }),
            None,
        );

        // TODO(roscale): I should probably register the receiver on the event loop instead of
        // blocking. It's usually fast, but sometimes it takes up to 1-2 ms.
        // Can this be solved by setting thread priorities?
        let handled = key_handled_rx.recv();
        Ok(handled?)
    }
}

impl<UserData> Drop for Engine<UserData> {
    fn drop(&mut self) {
        let result = unsafe { FlutterEngineShutdown(*self.handle) };

        if result != 0 {
            eprintln!("Failed to shut down Flutter engine, error {result}");
        }
    }
}

pub enum KeyState {
    Down,
    Up,
    Repeat,
}

fn create_view_metrics(width: usize, height: usize, view_id: i64) -> FlutterWindowMetricsEvent {
    FlutterWindowMetricsEvent {
        struct_size: size_of::<FlutterWindowMetricsEvent>(),
        width,
        height,
        pixel_ratio: 1.0,
        left: 0,
        top: 0,
        physical_view_inset_top: 0.0,
        physical_view_inset_right: 0.0,
        physical_view_inset_bottom: 0.0,
        physical_view_inset_left: 0.0,
        display_id: 0,
        view_id,
    }
}

fn get_bundle_path(flutter_project: &Path, aot_mode: bool) -> PathBuf {
    let mut path = flutter_project.join("build").join("linux");

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        panic!("Unsupported architecture");
    };

    path = path
        .join(arch)
        .join(if aot_mode { "release" } else { "debug" })
        .join("bundle");

    absolute(path).expect("Can't get absolute path to Flutter project bundle")
}
