use crate::flutter_engine_rs::flutter_engine_sys::{
    FlutterAddViewResult, FlutterBackingStore, FlutterBackingStore__bindgen_ty_1,
    FlutterBackingStoreConfig, FlutterBackingStoreType_kFlutterBackingStoreTypeOpenGL,
    FlutterOpenGLBackingStore, FlutterOpenGLBackingStore__bindgen_ty_1,
    FlutterOpenGLTargetType_kFlutterOpenGLTargetTypeTexture, FlutterOpenGLTexture,
    FlutterPlatformMessage, FlutterPresentViewInfo, FlutterTask,
};
use crate::flutter_engine_rs::platform_channels::binary_messenger::BinaryMessenger;
use crate::flutter_engine_rs::task_runner::{ScheduledTask, Task};
use crate::flutter_engine_rs::user_data::{EngineUserData, TaskRunnerUserData};
use std::ffi::{CStr, c_char, c_void};
use std::ptr::null_mut;

pub unsafe extern "C" fn runs_task_on_current_thread_callback(user_data: *mut c_void) -> bool {
    let user_data: &mut TaskRunnerUserData =
        unsafe { &mut *(user_data as *mut TaskRunnerUserData) };

    user_data.thread_id == std::thread::current().id()
}

pub unsafe extern "C" fn post_task_callback(
    task: FlutterTask,
    target_time: u64,
    user_data: *mut c_void,
) {
    let user_data: &mut TaskRunnerUserData =
        unsafe { &mut *(user_data as *mut TaskRunnerUserData) };

    user_data
        .tasks_tx
        .send(ScheduledTask {
            task: Task::FlutterTask(task),
            target_time,
        })
        .expect("Failed to send task");
}

pub unsafe extern "C" fn gl_proc_resolver_callback<UserData>(
    user_data: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    let c_str = unsafe { CStr::from_ptr(name) };
    let name = c_str.to_str().unwrap();

    (user_data.proc_resolver_callback)(name, &mut user_data.user_data) as *mut c_void
}

pub unsafe extern "C" fn vsync_callback<UserData>(user_data: *mut c_void, baton: isize) {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.vsync_callback)(baton, &mut user_data.user_data);
}

pub unsafe extern "C" fn make_current_callback<UserData>(user_data: *mut c_void) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.make_current_callback)(&mut user_data.user_data)
}

pub unsafe extern "C" fn clear_current_callback<UserData>(user_data: *mut c_void) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.clear_current_callback)(&mut user_data.user_data)
}

pub unsafe extern "C" fn present(_user_data: *mut c_void) -> bool {
    unreachable!("Unused we're using the FlutterCompositor");
}

pub unsafe extern "C" fn fbo_callback(_user_data: *mut c_void) -> u32 {
    unreachable!("Unused because we're using the FlutterCompositor");
}

pub unsafe extern "C" fn make_resource_current_callback<UserData>(user_data: *mut c_void) -> bool {
    let user_data: &mut EngineUserData<UserData> =
        unsafe { &mut *(user_data as *mut EngineUserData<UserData>) };

    (user_data.make_resource_current_callback)(&mut user_data.user_data)
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

    let name = (user_data.request_texture_callback)(
        backing_store_config.view_id,
        backing_store_config.size.width as u32,
        backing_store_config.size.height as u32,
        &mut user_data.user_data,
    );

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
                        // I don't want to import the OpenGL header just for a single macro.
                        target: 0x0DE1, // GL_TEXTURE_2D
                        name,
                        format: 0x8058, // GL_RGBA8
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

    (user_data.present_callback)(present_view_info.view_id, &mut user_data.user_data);

    true
}

pub unsafe extern "C" fn log_message_callback(
    tag: *const std::os::raw::c_char,
    message: *const std::os::raw::c_char,
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
