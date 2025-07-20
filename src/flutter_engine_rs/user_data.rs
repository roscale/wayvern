use std::sync::mpsc;
use std::thread::ThreadId;

use crate::flutter_engine_rs::{
    binary_messenger_impl::BinaryMessengerImpl,
    engine_callbacks::{
        ClearCurrentCallback, MakeCurrentCallback, MakeResourceCurrentCallback, PresentCallback,
        ProcResolverCallback, RequestTextureCallback, VSyncCallback,
    },
    task_runner::ScheduledTask,
};

pub struct TaskRunnerUserData {
    pub thread_id: ThreadId,
    pub tasks_tx: mpsc::Sender<ScheduledTask>,
}

pub struct EngineUserData<UserData> {
    pub proc_resolver_callback: ProcResolverCallback<UserData>,
    pub vsync_callback: VSyncCallback<UserData>,
    pub make_current_callback: MakeCurrentCallback<UserData>,
    pub clear_current_callback: ClearCurrentCallback<UserData>,
    pub make_resource_current_callback: MakeResourceCurrentCallback<UserData>,
    pub present_callback: PresentCallback<UserData>,
    pub request_texture_callback: RequestTextureCallback<UserData>,
    pub binary_messenger: BinaryMessengerImpl,
    pub user_data: UserData,
}
