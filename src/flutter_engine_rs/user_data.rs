use crate::flutter_engine_rs::{
    binary_messenger_impl::BinaryMessengerImpl,
    engine_callbacks::{
        ClearCurrentCallback, ExternalTextureFrameCallback, MakeCurrentCallback, MakeResourceCurrentCallback, PresentCallback, ProcResolverCallback, RequestTextureCallback, VSyncCallback
    },
};

pub struct EngineUserData<UserData> {
    pub proc_resolver: ProcResolverCallback<UserData>,
    pub external_texture_frame: ExternalTextureFrameCallback<UserData>,
    pub vsync: VSyncCallback<UserData>,
    pub make_current: MakeCurrentCallback<UserData>,
    pub clear_current: ClearCurrentCallback<UserData>,
    pub make_resource_current: MakeResourceCurrentCallback<UserData>,
    pub present: PresentCallback<UserData>,
    pub request_texture: RequestTextureCallback<UserData>,
    pub binary_messenger: BinaryMessengerImpl,
    pub user_data: UserData,
}
