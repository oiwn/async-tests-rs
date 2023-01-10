use async_trait::async_trait;
use rand::prelude::*;
use simple_logger::SimpleLogger;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

#[async_trait]
pub trait JobUnit {
    type Task;
    type Output;

    async fn run(self, task: Self::Task) -> Self::Output;
}

async fn run(value: u64) -> u64 {
    log::info!("[{}] Started", value);
    let range = rand::thread_rng().gen_range(0..10);
    sleep(Duration::from_secs(range)).await;
    log::info!("[{}] Finished", value);
    value
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();
    // let (send, mut recv) = channel(2);

    let mut set = JoinSet::new();
    for i in 0..10 {
        set.spawn(async move {
            // Do some async work
            let r = run(i).await;
            format!("{}", r)
        });
    }

    while let Some(res) = set.join_next().await {
        let out = res;
        println!("GOT {:?}", out);
    }

    //
    // Wait for the tasks to finish.
    //
    // We drop our sender first because the recv() call otherwise
    // sleeps forever.
    // drop(send);

    // When every sender has gone out of scope, the recv call
    // will return with an error. We ignore the error.
    // let _ = recv.recv().await;
}
