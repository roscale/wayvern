mod engine_callbacks;
mod engine_handle;
mod ffi;
mod flutter_engine_sys;
mod task_runner;
mod user_data;

use anyhow::{Result, ensure};
use core::fmt;
pub use engine_callbacks::EngineCallbacks;
use engine_handle::EngineHandle;
use flutter_engine_sys::*;
use std::{
    ffi::{CString, c_int, c_void},
    mem::size_of,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf, absolute},
    ptr::{null, null_mut},
};
use user_data::EngineUserData;

pub struct Engine<UserData> {
    handle: EngineHandle,
    _engine_user_data: Box<EngineUserData<UserData>>,
    _platform_task_runner: task_runner::TaskRunner,
    _render_task_runner: task_runner::TaskRunner,
    _ui_task_runner: task_runner::TaskRunner,
}

impl<UserData> Engine<UserData> {
    pub fn new(
        flutter_project: &Path,
        aot_mode: bool,
        callbacks: EngineCallbacks<UserData>,
        user_data: UserData,
    ) -> Result<Self> {
        let EngineCallbacks {
            proc_resolver: proc_resolver_callback,
            make_current: make_current_callback,
            clear_current: clear_current_callback,
            make_resource_current: make_resource_current_callback,
            request_texture: request_texture_callback,
            vsync: vsync_callback,
            present: present_callback,
            platform_message_callback,
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

        let engine_user_data = Box::new(user_data::EngineUserData {
            proc_resolver_callback,
            platform_message_callback,
            vsync_callback,
            make_current_callback,
            clear_current_callback,
            make_resource_current_callback,
            present_callback,
            request_texture_callback,
            user_data,
        });

        let platform_task_runner = task_runner::spawn_task_runner();
        let render_task_runner = task_runner::spawn_task_runner();
        let ui_task_runner = task_runner::spawn_task_runner();

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
                            gl_external_texture_frame_callback: None,
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
                            user_data: &*platform_task_runner.user_data as *const _ as *mut _,
                            runs_task_on_current_thread_callback: Some(
                                ffi::runs_task_on_current_thread_callback,
                            ),
                            post_task_callback: Some(ffi::post_task_callback),
                            identifier: 0,
                            destruction_callback: None,
                        },
                        render_task_runner: &FlutterTaskRunnerDescription {
                            struct_size: size_of::<FlutterTaskRunnerDescription>(),
                            user_data: &*render_task_runner.user_data as *const _ as *mut _,
                            runs_task_on_current_thread_callback: Some(
                                ffi::runs_task_on_current_thread_callback,
                            ),
                            post_task_callback: Some(ffi::post_task_callback),
                            identifier: 1,
                            destruction_callback: None,
                        },
                        thread_priority_setter: None,
                        ui_task_runner: &FlutterTaskRunnerDescription {
                            struct_size: size_of::<FlutterTaskRunnerDescription>(),
                            user_data: &*render_task_runner.user_data as *const _ as *mut _,
                            runs_task_on_current_thread_callback: Some(
                                ffi::runs_task_on_current_thread_callback,
                            ),
                            post_task_callback: Some(ffi::post_task_callback),
                            identifier: 2,
                            destruction_callback: None,
                        },
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

        ensure!(
            result == 0,
            "Could not initalize the Flutter engine, error {result}"
        );

        platform_task_runner
            .engine_handle_tx
            .send(EngineHandle(handle))
            .expect("Failed to send Flutter engine handle to task runner");

        render_task_runner
            .engine_handle_tx
            .send(EngineHandle(handle))
            .expect("Failed to send Flutter engine handle to task runner");

        ui_task_runner
            .engine_handle_tx
            .send(EngineHandle(handle))
            .expect("Failed to send Flutter engine handle to task runner");

        let result = unsafe { FlutterEngineRunInitialized(handle) };

        ensure!(
            result == 0,
            "Could not run the Flutter engine, error {result}"
        );

        let flutter_engine = Self {
            handle: EngineHandle(handle),
            _engine_user_data: engine_user_data,
            _platform_task_runner: platform_task_runner,
            _render_task_runner: render_task_runner,
            _ui_task_runner: ui_task_runner,
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

        let result = unsafe { FlutterEngineAddView(*self.handle, &info) };

        ensure!(result == 0, "Could add view {view_id}, error {result}");

        Ok(())
    }

    pub fn send_view_metrics(&mut self, view_id: i64, width: usize, height: usize) -> Result<()> {
        let view_metrics = create_view_metrics(width, height, view_id);

        let result =
            unsafe { FlutterEngineSendWindowMetricsEvent(*self.handle, &view_metrics as *const _) };

        ensure!(
            result == 0,
            "Could not send window metrics event, error {result}"
        );

        Ok(())
    }

    pub fn get_current_time(&self) -> u64 {
        unsafe { FlutterEngineGetCurrentTime() }
    }

    pub fn on_vsync(
        &self,
        baton: isize,
        frame_start_time_nanos: u64,
        frame_target_time_nanos: u64,
    ) -> Result<()> {
        let result = unsafe {
            FlutterEngineOnVsync(
                *self.handle,
                baton,
                frame_start_time_nanos,
                frame_target_time_nanos,
            )
        };

        ensure!(result == 0, "Could not send vsync baton, error {result}");

        Ok(())
    }

    pub fn register_external_texture(&self, texture_id: i64) -> Result<()> {
        let result = unsafe { FlutterEngineRegisterExternalTexture(*self.handle, texture_id) };

        ensure!(
            result == 0,
            "Could not register external texture, error {result}"
        );

        Ok(())
    }

    pub fn unregister_external_texture(&self, texture_id: i64) -> Result<()> {
        let result = unsafe { FlutterEngineUnregisterExternalTexture(*self.handle, texture_id) };

        ensure!(
            result == 0,
            "Could not unregister external texture, error {result}"
        );

        Ok(())
    }

    pub fn mark_external_texture_frame_available(&self, texture_id: i64) -> Result<()> {
        let result =
            unsafe { FlutterEngineMarkExternalTextureFrameAvailable(*self.handle, texture_id) };

        ensure!(
            result == 0,
            "Could not mark external texture frame available, error {result}"
        );

        Ok(())
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
