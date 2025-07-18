use std::ffi::c_void;

use crate::flutter_engine_rs::flutter_engine_sys::FlutterPlatformMessage;

pub type ProcResolverCallback<UserData> = Box<dyn Fn(&str, &mut UserData) -> *const c_void>;
pub type MakeCurrentCallback<UserData> = Box<dyn Fn(&mut UserData) -> bool + Send>;
pub type ClearCurrentCallback<UserData> = Box<dyn Fn(&mut UserData) -> bool + Send>;
pub type MakeResourceCurrentCallback<UserData> = Box<dyn Fn(&mut UserData) -> bool + Send>;
pub type RequestTextureCallback<UserData> = Box<dyn Fn(i64, u32, u32, &mut UserData) -> u32 + Send>;
pub type VSyncCallback<UserData> = Box<dyn Fn(isize, &mut UserData) + Send>;
pub type PresentCallback<UserData> = Box<dyn Fn(i64, &mut UserData) + Send>;
pub type PlatformMessageCallback<UserData> =
    Box<dyn Fn(&FlutterPlatformMessage, &mut UserData) + Send>;

pub struct EngineCallbacks<UserData> {
    pub proc_resolver: ProcResolverCallback<UserData>,
    pub make_current: MakeCurrentCallback<UserData>,
    pub clear_current: ClearCurrentCallback<UserData>,
    pub make_resource_current: MakeResourceCurrentCallback<UserData>,
    pub request_texture: RequestTextureCallback<UserData>,
    pub vsync: VSyncCallback<UserData>,
    pub present: PresentCallback<UserData>,
    pub platform_message_callback: PlatformMessageCallback<UserData>,
}
