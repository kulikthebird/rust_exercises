use futures::future::Future;
use log::debug;
use std::pin::Pin;
use std::sync::mpsc::Sender;
use std::task::{RawWaker, RawWakerVTable, Waker};

pub type TaskID = u64;

pub struct Task<'a> {
    pub future: Pin<Box<dyn Future<Output = ()> + 'a>>,
    task_sender: Sender<TaskID>,
    pub leak_box: bool,
}

impl<'a> Task<'a> {
    pub fn from_boxed_future(
        future: Pin<Box<dyn Future<Output = ()> + 'a>>,
        task_sender: Sender<TaskID>,
        leak_box: bool,
    ) -> Task<'a> {
        Task {
            future,
            task_sender,
            leak_box,
        }
    }

    pub unsafe fn new_waker(&self) -> Waker {
        let ptr = std::mem::transmute(self);
        Waker::from_raw(RawWaker::new(ptr, &TASK_VTABLE))
    }

    pub fn generate_task_id(&self) -> TaskID {
        let ptr = &*self.future as *const dyn Future<Output = ()>;
        let ptr = ptr as *const ();
        let ptr = ptr as TaskID;
        ptr
    }
}

impl<'a> Drop for Task<'a> {
    fn drop(&mut self) {
        let temp_future: Box<dyn Future<Output = ()>> = Box::new(async {});
        let mut temp_future = unsafe { Pin::new_unchecked(temp_future) };
        std::mem::swap(&mut self.future, &mut temp_future);
        if self.leak_box {
            Box::into_raw(Box::new(temp_future));
        }
    }
}

unsafe fn wake_task(input: *const ()) {
    debug!("Task is waking up.");
    let task: &Task = std::mem::transmute(input);
    let task_id = task.generate_task_id();
    debug!("Task ID = {}", task_id);
    task.task_sender
        .send(task_id)
        .expect("Not enough space in the queue.");
}

fn clone_task(task: *const ()) -> RawWaker {
    RawWaker::new(task, &TASK_VTABLE)
}

fn wake_by_ref_task(_task: *const ()) {}
fn drop_task(_task: *const ()) {}

lazy_static::lazy_static! {
    pub static ref TASK_VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone_task, wake_task, wake_by_ref_task, drop_task);
}
