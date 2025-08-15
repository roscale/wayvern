use tracing::error;

use crate::flutter_engine_rs::flutter_engine_sys::{
    FlutterAddViewResult, FlutterBackingStore, FlutterBackingStore__bindgen_ty_1,
    FlutterBackingStoreConfig, FlutterBackingStoreType_kFlutterBackingStoreTypeOpenGL,
    FlutterOpenGLBackingStore, FlutterOpenGLBackingStore__bindgen_ty_1,
    FlutterOpenGLTargetType_kFlutterOpenGLTargetTypeTexture, FlutterOpenGLTexture,
    FlutterPlatformMessage, FlutterPresentViewInfo, FlutterTask,
};
use crate::flutter_engine_rs::platform_channels::binary_messenger::BinaryMessenger;
use crate::flutter_engine_rs::task_runner::TaskPoster;
use crate::flutter_engine_rs::user_data::EngineUserData;
use std::ffi::{CStr, c_char, c_void};
use std::ptr::null_mut;
use std::sync::mpsc;

// I don't want to import the OpenGL header just for a few constants.
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_RGBA8: u32 = 0x8058;

pub unsafe extern "C" fn runs_task_on_current_thread_callback(user_data: *mut c_void) -> bool {
    let task_poster: &mut TaskPoster = unsafe { &mut *(user_data as *mut TaskPoster) };

    task_poster.runs_task_on_current_thread()
}

pub unsafe extern "C" fn post_task_callback(
    task: FlutterTask,
    target_time: u64,
    user_data: *mut c_void,
) {
    let task_poster: &mut TaskPoster = unsafe { &mut *(user_data as *mut TaskPoster) };

    task_poster.post_flutter_task(task, target_time);
}

pub unsafe extern "C" fn gl_proc_resolver_callback<UserData>(
    user_data: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    let c_str = unsafe { CStr::from_ptr(name) };
    let name = c_str.to_str().unwrap();

    (user_data.proc_resolver)(name, &mut user_data.user_data) as *mut c_void
}

pub unsafe extern "C" fn gl_external_texture_frame_callback<UserData>(
    user_data: *mut c_void,
    texture_id: i64,
    _width: usize,
    _height: usize,
    texture_out: *mut FlutterOpenGLTexture,
) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    let texture_out = unsafe { &mut *texture_out };

    // TODO(roscale): We should send width/height too and check if they are consistent.
    let texture_name = (user_data.external_texture_frame)(texture_id, &mut user_data.user_data);

    if let Some(texture_name) = texture_name {
        texture_out.target = GL_TEXTURE_2D;
        texture_out.name = texture_name;
        texture_out.format = GL_RGBA8;

        true
    } else {
        false
    }
}

pub unsafe extern "C" fn vsync_callback<UserData>(user_data: *mut c_void, baton: isize) {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.vsync)(baton, &mut user_data.user_data);
}

pub unsafe extern "C" fn make_current_callback<UserData>(user_data: *mut c_void) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.make_current)(&mut user_data.user_data)
}

pub unsafe extern "C" fn clear_current_callback<UserData>(user_data: *mut c_void) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.clear_current)(&mut user_data.user_data)
}

pub unsafe extern "C" fn present(_user_data: *mut c_void) -> bool {
    unreachable!(
        "Unused because we're using the FlutterCompositor, but it needs to be implemented"
    );
}

pub unsafe extern "C" fn fbo_callback(_user_data: *mut c_void) -> u32 {
    unreachable!(
        "Unused because we're using the FlutterCompositor, but it needs to be implemented"
    );
}

pub unsafe extern "C" fn make_resource_current_callback<UserData>(user_data: *mut c_void) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.make_resource_current)(&mut user_data.user_data)
}

pub unsafe extern "C" fn platform_message_callback<UserData>(
    message: *const FlutterPlatformMessage,
    user_data: *mut c_void,
) {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    user_data
        .binary_messenger
        .handle_message(unsafe { &*message });
}

pub unsafe extern "C" fn create_backing_store_callback<UserData>(
    backing_store_config: *const FlutterBackingStoreConfig,
    backing_store_out: *mut FlutterBackingStore,
    user_data: *mut c_void,
) -> bool {
    let user_data = unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };
    let backing_store_config = unsafe { &*backing_store_config };

    let name = (user_data.request_texture)(
        backing_store_config.view_id,
        backing_store_config.size.width as u32,
        backing_store_config.size.height as u32,
        &mut user_data.user_data,
    )
    .unwrap_or(0);

    let backing_store = FlutterBackingStore {
        struct_size: std::mem::size_of::<FlutterBackingStore>(),
        did_update: false,
        type_: FlutterBackingStoreType_kFlutterBackingStoreTypeOpenGL,
        user_data: user_data as *mut _ as *mut c_void,
        __bindgen_anon_1: FlutterBackingStore__bindgen_ty_1 {
            open_gl: FlutterOpenGLBackingStore {
                type_: FlutterOpenGLTargetType_kFlutterOpenGLTargetTypeTexture,
                __bindgen_anon_1: FlutterOpenGLBackingStore__bindgen_ty_1 {
                    texture: FlutterOpenGLTexture {
                        target: GL_TEXTURE_2D,
                        name,
                        format: GL_RGBA8,
                        user_data: null_mut(),
                        destruction_callback: None,
                        width: 0,
                        height: 0,
                    },
                },
            },
        },
    };

    unsafe {
        *backing_store_out = backing_store;
    }

    true
}

pub unsafe extern "C" fn collect_backing_store_callback(
    _backing_store: *const FlutterBackingStore,
    _user_data: *mut c_void,
) -> bool {
    true
}

pub unsafe extern "C" fn present_view_callback<UserData>(
    present_view_info: *const FlutterPresentViewInfo,
) -> bool {
    let present_view_info = unsafe { &*present_view_info };
    let user_data = unsafe { &mut *(present_view_info.user_data as *mut EngineUserData<UserData>) };

    (user_data.present)(present_view_info.view_id, &mut user_data.user_data);

    true
}

pub unsafe extern "C" fn log_message_callback(
    tag: *const c_char,
    message: *const c_char,
    _user_data: *mut c_void,
) {
    let tag = unsafe { CStr::from_ptr(tag).to_string_lossy() };
    let message = unsafe { CStr::from_ptr(message).to_string_lossy() };

    println!("[{tag}] {message}");
}

pub unsafe extern "C" fn add_view_callback(result: *const FlutterAddViewResult) {
    let result = unsafe { &*result };

    let callback = unsafe { Box::from_raw(result.user_data as *mut Box<dyn Fn(bool)>) };
    callback(result.added);
}

pub unsafe extern "C" fn key_event_callback(handled: bool, user_data: *mut c_void) {
    let tx = unsafe { Box::from_raw(user_data as *mut mpsc::Sender<bool>) };

    if let Err(err) = tx.send(handled) {
        error!("Failed to return handled key state, {err}");
    }
}
