mod async_timer;
mod executor;
use std::time::Duration;

fn main() {
    env_logger::init();
    let mut executor = executor::Executor::new();
    executor.spawn(async {
        println!("Start timer 1!");
        async_timer::TimerFuture::new(Duration::new(3, 0)).await;
        println!("End timer 1!");
    });
    executor.spawn(async {
        println!("Start timer 2!");
        async_timer::TimerFuture::new(Duration::new(2, 0)).await;
        println!("End timer 2!");
    });
    executor.run();
    executor.block_on(async {
        println!("Start timer 3!");
        async_timer::TimerFuture::new(Duration::new(2, 0)).await;
        println!("End timer 3!");
    });

    for i in 1..8100 {
        executor.spawn(async move {
            async_timer::TimerFuture::new(Duration::new(2, i*10)).await;
        });
    }
    executor.run();
}
