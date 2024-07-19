use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};

use crossbeam_queue::ArrayQueue;

use super::{Task, TaskId};

const TASK_QUEUE_SIZE: usize = 100;

// TODO: Utilize CPU threads with load balancing.
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    // Interrupt handlers should not allocate on push to this queue, so it's fixed size
    // TODO: Prioritize latency-critical tasks or tasks that do a lot of I/O (Scheduling).
    task_queue: Arc<ArrayQueue<TaskId>>,
    // Also, ensures that reference-counted wakers are not deallocated inside interrupt handlers
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(TASK_QUEUE_SIZE)),
            waker_cache: BTreeMap::new(),
        }
    }

    // TODO: Create an additional Spawner type that shares some kind of queue with the executor
    // and allows task creation from within tasks themselves.
    // Since `spawn` no longer available after invoking the `run` method.
    #[expect(clippy::missing_panics_doc)]
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;

        // PERF: Do we really need to check for duplicated task ids?
        assert!(
            self.tasks.insert(task.id, task).is_none(),
            "task with same ID already in tasks"
        );

        self.task_queue.push(task_id).expect("task_queue full");
    }

    fn run_ready_tasks(&mut self) {
        while let Some(task_id) = self.task_queue.pop() {
            // Since a wake-up might occurs for a task that no longer exists
            let Some(task) = self.tasks.get_mut(&task_id) else {
                continue; // task no longer exists
            };

            let waker = self.waker_cache.entry(task_id).or_insert_with(|| {
                Waker::from(Arc::new(TaskWaker::new(
                    task_id,
                    Arc::<ArrayQueue<TaskId>>::clone(&self.task_queue),
                )))
            });

            let mut context = Context::from_waker(waker);

            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    self.tasks.remove(&task_id);
                    self.waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();

            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();

        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    const fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Self {
        Self {
            task_id,
            task_queue,
        }
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
