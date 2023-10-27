use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use smithay::reexports::calloop::timer::Timer;

use crate::flutter_engine::embedder::{FlutterEngine as FlutterEngineHandle, FlutterEngineGetCurrentTime, FlutterEngineRunTask, FlutterTask};

type TargetTime = u64;

#[derive(Copy, Clone, Eq, PartialEq)]
struct Task(FlutterTask, TargetTime);

impl PartialEq<Self> for FlutterTask {
    fn eq(&self, other: &Self) -> bool {
        self.task == other.task
    }
}

impl Eq for FlutterTask {}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip the ordering so that the BinaryHeap becomes a min-heap.
        // Smaller target times have higher priority.
        // In case of a tie we compare the task ID.
        // This step is necessary to make implementations of `PartialEq` and `Ord` consistent.
        other.1.cmp(&self.1).then_with(|| self.0.task.cmp(&other.0.task))
    }
}

impl PartialOrd<Self> for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TaskRunner {
    tasks: Mutex<BinaryHeap<Task>>,
    expired_tasks: Vec<Task>,
    timer: Timer,
}

impl TaskRunner {
    pub fn enqueue_task(&mut self, task: FlutterTask, target_time: TargetTime) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(Task(task, target_time));
        Self::schedule_timer(&mut self.timer, tasks);
    }

    pub fn execute_expired_tasks(&mut self, flutter_engine_handle: FlutterEngineHandle) {
        assert!(self.expired_tasks.is_empty());

        {
            let mut tasks = self.tasks.lock().unwrap();
            while let Some(task) = tasks.peek() {
                let Task(_, target_time) = *task;
                if target_time > unsafe { FlutterEngineGetCurrentTime() } {
                    // Target time is in the future.
                    break;
                }
                self.expired_tasks.push(tasks.pop().unwrap());
            }
            Self::schedule_timer(&mut self.timer, tasks);
        }

        for Task(task, _) in self.expired_tasks.drain(..) {
            unsafe { FlutterEngineRunTask(flutter_engine_handle, &task) };
        }
    }

    fn schedule_timer(timer: &mut Timer, tasks: MutexGuard<BinaryHeap<Task>>) {
        if let Some(Task(_, target_time)) = tasks.peek() {
            let now = unsafe { FlutterEngineGetCurrentTime() };
            let duration_ns = target_time.checked_sub(now).unwrap_or(0);
            timer.set_duration(Duration::from_nanos(duration_ns));
        }
    }
}
