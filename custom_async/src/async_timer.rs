use crate::executor::subscribe_for_wake;
use std::future::Future;
use std::os::unix::prelude::AsRawFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use timerfd::{SetTimeFlags, TimerFd, TimerState};
use log::debug;

pub struct TimerFuture {
    timer_fd: TimerFd,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        debug!("Poll the timer.");
        if self.timer_fd.get_state() == TimerState::Disarmed {
            Poll::Ready(())
        } else {
            subscribe_for_wake(cx.waker().clone(), self.timer_fd.as_raw_fd());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        let mut timer_fd = TimerFd::new().unwrap();
        timer_fd.set_state(TimerState::Oneshot(duration), SetTimeFlags::Default);
        TimerFuture { timer_fd }
    }
}
