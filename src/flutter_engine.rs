use std::ffi::{c_int, c_void, CString};
use std::mem::size_of;
use std::os::unix::ffi::OsStrExt;
use std::ptr::{null, null_mut};

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

use crate::{
    flutter_engine::{
        callbacks::{
            clear_current,
            fbo_callback,
            make_current,
            make_resource_current,
            present_with_info,
            surface_transformation,
        },
        embedder::{
            FLUTTER_ENGINE_VERSION,
            FlutterEngine as FlutterEngineInternal,
            FlutterEngineGetCurrentTime,
            FlutterEngineOnVsync,
            FlutterEngineRun,
            FlutterEngineSendWindowMetricsEvent,
            FlutterEngineShutdown,
            FlutterOpenGLRendererConfig,
            FlutterProjectArgs,
            FlutterRendererConfig,
            FlutterRendererConfig__bindgen_ty_1,
            FlutterWindowMetricsEvent,
        },
    },
};
use crate::flutter_engine::embedder::{FlutterEngineSendPointerEvent, FlutterPointerEvent};
use crate::gles_framebuffer_importer::GlesFramebufferImporter;

mod callbacks;
pub mod embedder;

/// Wrap the handle for various safety reasons:
/// - Clone & Copy is willingly not implemented to avoid using the engine after being shut down.
/// - Send is not implemented because all its methods must be called from the thread the engine was created.
pub struct FlutterEngine(FlutterEngineInternal, *mut FlutterEngineData);

/// I don't want people to clone it because it's UB to call [FlutterEngine::on_vsync] multiple times
/// with the same baton, which will most probably segfault.
pub struct Baton(isize);

impl FlutterEngine {
    pub fn new(root_egl_context: &EGLContext) -> Result<(Self, EmbedderChannels), Box<dyn std::error::Error>> {
        let (tx_present, rx_present) = channel::channel::<()>();
        let (tx_request_rbo, rx_request_rbo) = channel::channel::<()>();
        let (tx_rbo, rx_rbo) = channel::channel::<Option<Dmabuf>>();
        let (tx_output_height, rx_output_height) = channel::channel::<u16>();
        let (tx_baton, rx_baton) = channel::channel::<Baton>();

        let flutter_engine_channels = FlutterEngineChannels {
            tx_present,
            tx_request_rbo,
            rx_rbo,
            rx_output_height,
            tx_baton,
        };

        let embedder_channels = EmbedderChannels {
            rx_present,
            rx_request_rbo,
            tx_rbo,
            tx_output_height,
            rx_baton,
        };

        let flutter_engine_data = Box::new(FlutterEngineData::new(root_egl_context, flutter_engine_channels)?);
        let flutter_engine_data = Box::into_raw(flutter_engine_data);

        let assets_path = CString::new(if option_env!("BUNDLE").is_some() { "data/flutter_assets" } else { "wayvern_flutter/build/linux/x64/debug/bundle/data/flutter_assets" })?;
        let icu_data_path = CString::new(if option_env!("BUNDLE").is_some() { "data/icudtl.dat" } else { "wayvern_flutter/build/linux/x64/debug/bundle/data/icudtl.dat" })?;
        let executable_path = CString::new(std::fs::canonicalize("/proc/self/exe")?.as_os_str().as_bytes())?;
        let observatory_port = CString::new("--observatory-port=12345")?;
        let disable_service_auth_codes = CString::new("--disable-service-auth-codes")?;

        let command_line_argv = [
            executable_path.as_ptr(),
            observatory_port.as_ptr(),
            disable_service_auth_codes.as_ptr(),
        ];

        let project_args = FlutterProjectArgs {
            struct_size: size_of::<FlutterProjectArgs>(),
            assets_path: assets_path.as_ptr(),
            main_path__unused__: null(),
            packages_path__unused__: null(),
            icu_data_path: icu_data_path.as_ptr(),
            command_line_argc: command_line_argv.len() as c_int,
            command_line_argv: command_line_argv.as_ptr(),
            platform_message_callback: None,
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
            vsync_callback: None,
            // vsync_callback: Some(vsync_callback),
            custom_dart_entrypoint: null(),
            custom_task_runners: null(),
            shutdown_dart_vm_when_done: true,
            compositor: null(),
            dart_old_gen_heap_size: 0,
            aot_data: null_mut(),
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
                    gl_external_texture_frame_callback: None,
                    fbo_with_frame_info_callback: None,
                    present_with_info: Some(present_with_info),
                    populate_existing_damage: None,
                }
            },
        };

        let mut flutter_engine: FlutterEngineInternal = null_mut();
        let result = unsafe {
            FlutterEngineRun(
                FLUTTER_ENGINE_VERSION as usize,
                &renderer_config,
                &project_args,
                flutter_engine_data as *mut c_void,
                &mut flutter_engine,
            )
        };

        if result != 0 {
            return Err(format!("Could not initalize the Flutter engine, error {result}").into());
        }

        Ok((Self(flutter_engine, flutter_engine_data), embedder_channels))
    }

    pub fn current_time_ns() -> u64 {
        unsafe { FlutterEngineGetCurrentTime() }
    }

    pub fn current_time_ms() -> u64 {
        unsafe { FlutterEngineGetCurrentTime() / 1000 }
    }

    pub fn send_window_metrics(&self, size: Size<u32, Physical>) -> Result<(), Box<dyn std::error::Error>> {
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

        let result = unsafe { FlutterEngineSendWindowMetricsEvent(self.0, &event as *const _) };
        if result != 0 {
            return Err(format!("Could not send window metrics event, error {result}").into());
        }
        Ok(())
    }

    pub fn on_vsync(&self, baton: Baton) -> Result<(), Box<dyn std::error::Error>> {
        let now = unsafe { FlutterEngineGetCurrentTime() };
        let next_frame = now + (1_000_000_000 / 144);
        let result = unsafe { FlutterEngineOnVsync(self.0, baton.0, now, next_frame) };
        if result != 0 {
            return Err(format!("Could not send vsync baton, error {result}").into());
        }
        Ok(())
    }

    pub fn send_pointer_event(&self, event: FlutterPointerEvent) -> Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { FlutterEngineSendPointerEvent(self.0, &event as *const _, 1) };
        if result != 0 {
            return Err(format!("Could not send pointer event, error {result}").into());
        }
        Ok(())
    }
}

impl Drop for FlutterEngine {
    fn drop(&mut self) {
        let _ = unsafe { FlutterEngineShutdown(self.0) };
        let _ = unsafe { Box::from_raw(self.1) };
    }
}

struct FlutterEngineData {
    pub gl: Gles2,
    pub main_egl_context: EGLContext,
    pub resource_egl_context: EGLContext,
    pub output_height: Option<u16>,
    channels: FlutterEngineChannels,
    framebuffer_importer: GlesFramebufferImporter,
}

// Ironically, EGLContext which contains EGLDisplay is Send, but EGLDisplay is not.
// This impl is not needed, but it's here to make it explicit that it's safe to send this struct
// to the Flutter render thread.
unsafe impl Send for FlutterEngineData {}

impl FlutterEngineData {
    pub fn new(root_egl_context: &EGLContext, channels: FlutterEngineChannels) -> Result<Self, Box<dyn std::error::Error>> {
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
            output_height: None,
            channels,
            framebuffer_importer: unsafe { GlesFramebufferImporter::new(egl_display.clone())? },
        })
    }
}

pub struct FlutterEngineChannels {
    pub tx_present: channel::Sender<()>,
    pub tx_request_rbo: channel::Sender<()>,
    pub rx_rbo: channel::Channel<Option<Dmabuf>>,
    pub rx_output_height: channel::Channel<u16>,
    pub tx_baton: channel::Sender<Baton>,
}

pub struct EmbedderChannels {
    pub rx_present: channel::Channel<()>,
    pub rx_request_rbo: channel::Channel<()>,
    pub tx_rbo: channel::Sender<Option<Dmabuf>>,
    pub tx_output_height: channel::Sender<u16>,
    pub rx_baton: channel::Channel<Baton>,
}
