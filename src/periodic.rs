/// Periodically run an async function using tokio's `Notify` for precise timing
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tokio::task;
use tokio::time::sleep;

/// Run an async function periodically. The duration is the lower limit of the waiting time between each invocation
/// of the function, which, can be exceeded if the function takes too long to run.
pub async fn run_periodically<E, F, FF>(d: Duration, f: FF) -> Result<(), E>
    where
        F: Future<Output=Result<(), E>>,
        FF: Fn() -> F
{
    let flag = Arc::new(Notify::new());
    let flag2 = flag.clone();

    let reverse_flag = Arc::new(Notify::new());
    let reverse_flag2 = reverse_flag.clone();

    task::spawn(async move {
        loop {
            flag2.notify_one();
            sleep(d).await;
            reverse_flag2.notified();
        }
    });

    loop {
        flag.notified().await;
        reverse_flag.notify_one();
        f().await?;
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::Mutex;

    use crate::periodic::run_periodically;

    #[tokio::test]
    async fn test_run_periodically() {
        let counter = Arc::new(Mutex::new(0));
        let _ = run_periodically(Duration::from_secs(1), || async {
            let counter = counter.clone();
            let mut counter_guard = counter.lock().await;
            *counter_guard += 1;
            println!("hi");
            if *counter_guard >= 5 { Err(()) } else { Ok(()) }
        }).await;
    }
}