use crate::task::{Task, TaskID};
use futures::future::{Future, FutureExt};
use log::debug;
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::pin::Pin;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Mutex;
use std::task::Waker;
use std::task::{Context, Poll};

lazy_static::lazy_static! {
    pub static ref MAIN_DESCRIPTOR: RawFd =
        epoll::create(false).expect("Could not create main descriptor");
    pub static ref WAKERS: Mutex<RefCell<HashMap::<RawFd, Waker>>> =
        Mutex::new(RefCell::new(HashMap::new()));
}

pub fn subscribe_for_wake(waker: Waker, tracked_fd: RawFd) {
    epoll::ctl(
        *MAIN_DESCRIPTOR,
        epoll::ControlOptions::EPOLL_CTL_ADD,
        tracked_fd,
        epoll::Event::new(epoll::Events::EPOLLIN, tracked_fd as u64),
    )
    .expect("Some problem with epoll");
    (*WAKERS)
        .lock()
        .expect("Poisoned Mutex")
        .borrow_mut()
        .insert(tracked_fd, waker);
}

pub struct Executor<'a> {
    task_sender: SyncSender<TaskID>,
    ready_queue: Receiver<TaskID>,
    pending_tasks: HashMap<TaskID, Box<Task<'a>>>,
}

impl<'a> Executor<'a> {
    pub fn new(tasks_queue_bound: usize) -> Executor<'a> {
        let (task_sender, ready_queue) = sync_channel(tasks_queue_bound);
        Executor {
            task_sender,
            ready_queue,
            pending_tasks: HashMap::new(),
        }
    }

    pub fn finish_scheduled_tasks(&mut self) {
        let mut events_buffer = [epoll::Event::new(epoll::Events::EPOLLIN, 0)];
        while self.pending_tasks.len() > 0 {
            while let Ok(task_id) = self.ready_queue.try_recv() {
                let task = self
                    .pending_tasks
                    .remove(&task_id)
                    .expect("There is no such a task.");
                self.process_task(task);
            }
            let read_events = epoll::wait(*MAIN_DESCRIPTOR, 1000, &mut events_buffer)
                .expect("Problem occured while waiting for main descriptor");
            for i in 0..read_events {
                let task_id = events_buffer[i].data as RawFd;
                let waker = (*WAKERS)
                    .lock()
                    .expect("Poisoned Mutex")
                    .borrow_mut()
                    .remove(&task_id)
                    .expect("the waker should be present here");
                waker.wake();
            }
        }
    }

    fn process_task(&mut self, mut task: Box<Task<'a>>) {
        debug!("Processing the task");
        let waker = unsafe { task.new_waker() };
        let context = &mut Context::from_waker(&waker);
        let task_id = task.generate_task_id();
        if let Poll::Ready(()) = Pin::new(&mut task.future).poll(context) {
            debug!("Task is ready, task_id = {}", task_id);
        } else {
            debug!("Task is pending, task_id = {}", task_id);
            self.pending_tasks.insert(task_id, task);
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
        let future = unsafe { Pin::new_unchecked(Box::from_raw(&mut future)) };
        let task = Box::new(Task::from_boxed_future(
            future,
            self.task_sender.clone(),
            true,
        ));
        self.process_task(task);
        self.finish_scheduled_tasks();
    }
}
