use std::collections::BinaryHeap;
use std::time::Duration;
use std::{cmp::Ordering, sync::mpsc};

use crate::flutter_engine_rs::engine_handle::EngineHandle;
use crate::flutter_engine_rs::flutter_engine_sys::{
    FlutterEngineGetCurrentTime, FlutterEngineRunTask, FlutterTask,
};
use crate::flutter_engine_rs::monotonic_time::get_monotonic_time;
use crate::flutter_engine_rs::user_data::TaskRunnerUserData;

pub enum Task {
    FlutterTask(FlutterTask),
    Callback(Box<dyn FnOnce()>),
}

unsafe impl Send for Task {}

pub struct ScheduledTask {
    pub task: Task,
    pub target_time: u64,
}

impl PartialEq<Self> for ScheduledTask {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for ScheduledTask {}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip the ordering so that the BinaryHeap becomes a min-heap.
        // Smaller target times have higher priority.
        // In case of a tie, we compare the task ID.
        // This step is necessary to make implementations of `PartialEq` and `Ord` consistent.
        other.target_time.cmp(&self.target_time)
    }
}

impl PartialOrd<Self> for ScheduledTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn get_next_task_due_time(tasks: &mut BinaryHeap<ScheduledTask>) -> Duration {
    match tasks.peek() {
        None => Duration::MAX,
        Some(ScheduledTask { target_time, .. }) => {
            let now = unsafe { FlutterEngineGetCurrentTime() };
            let duration_ns = target_time.checked_sub(now).unwrap_or(0);
            Duration::from_nanos(duration_ns)
        }
    }
}

fn drain_channel(
    rx: &mut mpsc::Receiver<ScheduledTask>,
    tasks: &mut BinaryHeap<ScheduledTask>,
) -> Result<(), ()> {
    loop {
        match rx.try_recv() {
            Ok(task) => {
                tasks.push(task);
            }
            Err(mpsc::TryRecvError::Empty) => return Ok(()),
            Err(mpsc::TryRecvError::Disconnected) => return Err(()),
        }
    }
}

pub struct TaskRunner {
    // tasks_tx must be declared before join_handle to prevent a deadlock.
    // Fields are dropped first to last.
    // If the sender is dropped, the thread will detect it and end on its own.
    // Then the join_handle can be dropped without deadlocking because the thread is already in the process of exiting.
    pub user_data: Box<TaskRunnerUserData>,
    _join_handle: jod_thread::JoinHandle,
    pub engine_handle_tx: mpsc::Sender<EngineHandle>,
}

pub fn spawn_task_runner() -> TaskRunner {
    let (tasks_tx, mut tasks_rx) = mpsc::channel();
    let (thread_id_tx, thread_id_rx) = mpsc::channel();
    let (flutter_engine_handle_tx, flutter_engine_handle_rx) = mpsc::channel::<EngineHandle>();

    let join_handle = jod_thread::spawn(move || {
        let thread_id = std::thread::current().id();

        thread_id_tx
            .send(thread_id)
            .expect("Failed to send thread ID");

        let Ok(flutter_engine_handle) = flutter_engine_handle_rx.recv() else {
            return;
        };
        let flutter_engine_handle = *flutter_engine_handle;

        let mut tasks = BinaryHeap::new();

        loop {
            let due_time = get_next_task_due_time(&mut tasks);

            if let Ok(task) = tasks_rx.recv_timeout(due_time) {
                tasks.push(task);
            }

            // Drain the channel if multiple tasks were enqueued.
            if drain_channel(&mut tasks_rx, &mut tasks).is_err() {
                return;
            }

            while let Some(task) = tasks.peek() {
                let now = unsafe { FlutterEngineGetCurrentTime() };

                if task.target_time > now {
                    break;
                }

                let task = tasks.pop().unwrap();

                match task.task {
                    Task::FlutterTask(flutter_task) => {
                        unsafe {
                            FlutterEngineRunTask(
                                flutter_engine_handle,
                                &flutter_task as *const FlutterTask,
                            )
                        };
                    }
                    Task::Callback(fn_once) => {
                        fn_once();
                    }
                }

                if drain_channel(&mut tasks_rx, &mut tasks).is_err() {
                    return;
                }
            }
        }
    });

    let thread_id = thread_id_rx.recv().expect("Failed to receive thread ID");

    TaskRunner {
        user_data: Box::new(TaskRunnerUserData {
            tasks_tx,
            thread_id,
        }),
        _join_handle: join_handle,
        engine_handle_tx: flutter_engine_handle_tx,
    }
}

pub trait PostCallback {
    fn post_callback<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + 'static,
        R: 'static;
}

impl PostCallback for mpsc::Sender<ScheduledTask> {
    fn post_callback<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + 'static,
        R: 'static,
    {
        let (tx_response, rx_response) = mpsc::channel();

        self.send(ScheduledTask {
            task: Task::Callback(Box::new(move || {
                tx_response.send(f()).unwrap();
            })),
            target_time: get_monotonic_time(),
        })
        .expect("Failed to send callback to platform thread");

        rx_response
            .recv()
            .expect("Failed to get response from channel")
    }
}
