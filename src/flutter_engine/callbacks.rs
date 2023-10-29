use std::ffi::c_void;

use tracing::error;

use crate::flutter_engine::{Baton, FlutterEngine};
use crate::flutter_engine::embedder::{FlutterDamage, FlutterPlatformMessage, FlutterPresentInfo, FlutterRect, FlutterTask, FlutterTransformation};
use crate::flutter_engine::platform_channels::binary_messenger::BinaryMessenger;

pub unsafe extern "C" fn make_current(user_data: *mut c_void) -> bool {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    match flutter_engine.data.main_egl_context.make_current() {
        Ok(()) => true,
        Err(err) => {
            error!("{}", err);
            false
        },
    }
}

pub unsafe extern "C" fn make_resource_current(user_data: *mut c_void) -> bool {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    match flutter_engine.data.resource_egl_context.make_current() {
        Ok(()) => true,
        Err(err) => {
            error!("{}", err);
            false
        },
    }
}

pub unsafe extern "C" fn clear_current(user_data: *mut c_void) -> bool {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    match flutter_engine.data.main_egl_context.unbind() {
        Ok(()) => true,
        Err(err) => {
            error!("{}", err);
            false
        },
    }
}

pub unsafe extern "C" fn fbo_callback(user_data: *mut c_void) -> u32 {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    if flutter_engine.data.channels.tx_request_fbo.send(()).is_err() {
        return 0;
    }
    if let Ok(Some(dmabuf)) = flutter_engine.data.channels.rx_fbo.recv() {
        flutter_engine.data.framebuffer_importer.import_framebuffer(&flutter_engine.data.main_egl_context, dmabuf).unwrap_or(0)
    } else {
        0
    }
}

pub unsafe extern "C" fn present_with_info(user_data: *mut c_void, _frame_present_info: *const FlutterPresentInfo) -> bool {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    flutter_engine.data.gl.Finish();
    flutter_engine.data.channels.tx_present.send(()).is_ok()
}

pub unsafe extern "C" fn populate_existing_damage(_user_data: *mut c_void, _fbo_id: isize, existing_damage: *mut FlutterDamage) {
    let existing_damage = &mut *existing_damage;
    existing_damage.struct_size = std::mem::size_of::<FlutterDamage>();
    existing_damage.num_rects = 1;

    // TODO: Implement proper damage tracking.
    // The Flutter engine docs says that if this callback is not implemented,
    // it will repaint the entire screen every frame, but it's false.
    // It's probably a bug in the Flutter engine, but until I implement proper damage tracking
    // we manually set the damage to the entire screen.
    static FLUTTER_RECT: FlutterRect = FlutterRect {
        left: 0.0,
        top: 0.0,
        right: 10000.0,
        bottom: 10000.0,
    };

    existing_damage.damage = &FLUTTER_RECT as *const _ as *mut _;
}

pub unsafe extern "C" fn surface_transformation(user_data: *mut c_void) -> FlutterTransformation {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);

    while let Ok(output_height) = flutter_engine.data.channels.rx_output_height.try_recv() {
        flutter_engine.data.output_height = Some(output_height);
    }

    match flutter_engine.data.output_height {
        Some(output_height) => FlutterTransformation {
            scaleX: 1.0,
            skewX: 0.0,
            transX: 0.0,
            skewY: 0.0,
            scaleY: -1.0,
            transY: output_height as f64,
            pers0: 0.0,
            pers1: 0.0,
            pers2: 1.0,
        },
        None => FlutterTransformation {
            scaleX: 1.0,
            skewX: 0.0,
            transX: 0.0,
            skewY: 0.0,
            scaleY: 1.0,
            transY: 0.0,
            pers0: 0.0,
            pers1: 0.0,
            pers2: 1.0,
        },
    }
}

pub unsafe extern "C" fn vsync_callback(user_data: *mut std::os::raw::c_void, baton: isize) {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    let _ = flutter_engine.data.channels.tx_baton.send(Baton(baton));
}

pub unsafe extern "C" fn runs_task_on_current_thread_callback(user_data: *mut c_void) -> bool {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    flutter_engine.current_thread_id == std::thread::current().id()
}

pub unsafe extern "C" fn post_task_callback(task: FlutterTask, target_time: u64, user_data: *mut c_void) {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    let timeout = flutter_engine.task_runner.enqueue_task(task, target_time);
    flutter_engine.task_runner.reschedule_timer.send(timeout).unwrap();
}

pub unsafe extern "C" fn platform_message_callback(message: *const FlutterPlatformMessage, user_data: *mut c_void) {
    let flutter_engine = &mut *(user_data as *mut FlutterEngine);
    let message = &*message;
    flutter_engine.binary_messenger.as_mut().unwrap().handle_message(message);
}

pub enum FlutterEngineIntent {
    RequestFramebuffer,
    PresentFramebuffer,
}

impl TryFrom<u32> for FlutterEngineIntent {
    type Error = ();

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == FlutterEngineIntent::RequestFramebuffer as u32 => Ok(FlutterEngineIntent::RequestFramebuffer),
            x if x == FlutterEngineIntent::PresentFramebuffer as u32 => Ok(FlutterEngineIntent::PresentFramebuffer),
            _ => Err(()),
        }
    }
}