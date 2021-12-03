mod async_timer;
mod executor;
mod task;
use std::time::{Duration, Instant};

fn main() {
    env_logger::init();
    let mut executor = executor::Executor::new();
    executor.spawn(async {
        let start = Instant::now();
        println!("Start timer 1!");
        async_timer::TimerFuture::new(Duration::new(3, 0)).await;
        println!("End timer 1! Elapsed: {:?}", start.elapsed());
    });
    executor.spawn(async {
        let start = Instant::now();
        println!("Start timer 2!");
        async_timer::TimerFuture::new(Duration::new(2, 0)).await;
        println!("End timer 2! Elapsed: {:?}", start.elapsed());
    });
    executor.finish_scheduled_tasks();
    executor.block_on(async {
        let start = Instant::now();
        println!("Start timer 3!");
        async_timer::TimerFuture::new(Duration::new(2, 0)).await;
        println!("End timer 3! Elapsed: {:?}", start.elapsed());
    });

    let start = Instant::now();
    println!("Start multiple timers!");
    for _ in 1..8100 {
        executor.spawn(async move {
            async_timer::TimerFuture::new(Duration::new(2, 0)).await;
        });
    }
    executor.finish_scheduled_tasks();
    println!("Multiple timers finished. Elapsed: {:?}", start.elapsed());

    let start = Instant::now();
    println!("Start multiple timers!");
    for _ in 1..8100 {
        executor.spawn(async move {
            async_timer::TimerFuture::new(Duration::new(2, 0)).await;
        });
    }
    executor.finish_scheduled_tasks();
    println!("Multiple timers finished. Elapsed: {:?}", start.elapsed());
}
