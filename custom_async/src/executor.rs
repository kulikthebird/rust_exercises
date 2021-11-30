use futures::future::{Future, FutureExt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::pin::Pin;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::task::{RawWaker, RawWakerVTable, Waker};
use log::debug;

struct Task<'a> {
    future: Pin<Box<dyn Future<Output = ()> + 'a>>,
    task_sender: SyncSender<usize>,
    leak_box: bool,
}

impl<'a> Task<'a> {
    pub fn from_boxed_future(
        future: Pin<Box<dyn Future<Output = ()> + 'a>>,
        task_sender: SyncSender<usize>,
        leak_box: bool,
    ) -> Task<'a> {
        Task {
            future,
            task_sender,
            leak_box,
        }
    }
}

unsafe fn new_waker_from_task(task_ptr: &Task) -> Waker {
    let ptr = std::mem::transmute(task_ptr);
    Waker::from_raw(RawWaker::new(ptr, &TASK_VTABLE))
}

fn generate_task_id<'a>(future: &Pin<Box<dyn Future<Output = ()> + 'a>>) -> usize {
    let ptr = &*future as *const dyn Future<Output = ()>;
    let ptr = ptr as *const ();
    let ptr = ptr as usize;
    ptr
}

unsafe fn wake_task(input: *const ()) {
    debug!("Task is waking up.");
    let task: &Task = std::mem::transmute(input);
    let task_id = generate_task_id(&task.future);
    debug!("Task ID = {}", task_id);
    task.task_sender
        .send(task_id)
        .expect("TODO: Not enough space in the queue.");
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

pub const MAX_QUEUED_TASKS: usize = 1000;

lazy_static::lazy_static! {
    pub static ref MAIN_DESCRIPTOR: RawFd = epoll::create(false).expect("TODO: Could not create main descriptor");
    pub static ref WAKERS: Mutex<RefCell<HashMap::<usize, Waker>>> = Mutex::new(RefCell::new(HashMap::new()));
}

pub fn subscribe_for_wake(waker: Waker, tracked_fd: RawFd) {
    epoll::ctl(
        *MAIN_DESCRIPTOR,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        tracked_fd,
        epoll::Event::new(epoll::Events::EPOLLIN, tracked_fd as u64),
    )
    .expect("TODO: Some problem with epoll");
    (*WAKERS)
        .lock()
        .expect("TODO: Poisoned Mutex")
        .borrow_mut()
        .insert(tracked_fd as usize, waker);
}

pub struct Executor<'a> {
    task_sender: SyncSender<usize>,
    ready_queue: Receiver<usize>,
    tasks: HashMap<usize, Box<Task<'a>>>,
}

impl<'a> Executor<'a> {
    pub fn new() -> Executor<'a> {
        let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
        Executor {
            task_sender,
            ready_queue,
            tasks: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let mut events_buffer = [epoll::Event::new(epoll::Events::EPOLLIN, 0); 1000];
        while self.tasks.len() > 0 {
            while let Ok(task_id) = self.ready_queue.try_recv() {
                let task = self
                    .tasks
                    .remove(&task_id)
                    .expect("TODO: There is no such a task.");
                self.process_task(task);
            }
            let read_events = epoll::wait(*MAIN_DESCRIPTOR, 1000, &mut events_buffer)
                .expect("TODO: Problem while waiting for main descriptor");
            for i in 0..read_events {
                let task_id = events_buffer[i].data as usize;
                let waker = (*WAKERS)
                    .lock()
                    .expect("TODO: Poisoned Mutex")
                    .borrow_mut()
                    .remove(&task_id)
                    .expect("TODO: the waker should be present here");
                waker.wake();
            }
        }
    }

    fn process_task(&mut self, mut task: Box<Task<'a>>) {
        debug!("Processing the task");
        let waker = unsafe { new_waker_from_task(&task) };
        let context = &mut Context::from_waker(&waker);
        let task_id = generate_task_id(&task.future);
        if let Poll::Ready(()) = Pin::new(&mut task.future).poll(context) {
            if task.leak_box {
                Box::leak(task.future.into());
            }
            debug!("Task is ready, task_id = {}", task_id);
        } else {
            debug!("Task is pending, task_id = {}", task_id);
            self.tasks.insert(task_id, task);
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        let task = Box::new(Task::from_boxed_future(
            future,
            self.task_sender.clone(),
            false,
        ));
        self.process_task(task);
    }

    pub fn block_on(&mut self, mut future: impl Future<Output = ()> + 'a) {
        let pinned_future = unsafe { Pin::new_unchecked(Box::from_raw(&mut future)) };
        let task = Box::new(Task::from_boxed_future(
            pinned_future,
            self.task_sender.clone(),
            true,
        ));
        self.process_task(task);
        self.run();
    }
}
