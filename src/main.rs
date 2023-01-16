use async_trait::async_trait;
use rand::prelude::*;
use simple_logger::SimpleLogger;
use tokio::signal;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

#[async_trait]
pub trait JobUnit {
    type Task;
    type Output;

    async fn run(self, task: Self::Task) -> Self::Output;
}

async fn run(
    value: u64,
    receiver: async_channel::Receiver<Option<Message>>,
    mut stop: broadcast::Receiver<bool>,
) -> u64 {
    log::info!("[{}] Started", value);
    loop {
        let msg = receiver.recv().await;
        match msg {
            Ok(m) => {
                if let Some(val) = m {
                    println!("Val: {}", val.num);
                } else {
                    break;
                }
            }
            Err(_) => break,
        };
        // println!("Message: {:?}", msg);
        let range = rand::thread_rng().gen_range(0..10);
        sleep(Duration::from_secs(range)).await;

        if let Ok(s) = stop.recv().await {
            if s == true {
                println!("Stop signal received...{}", s);
                break;
            }
        }
    }
    log::info!("[{}] Finished", value);
    value
}

#[derive(Debug)]
struct Message {
    num: u64,
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();
    let (send, recv) = async_channel::unbounded::<Option<Message>>();
    let (s, _) = broadcast::channel::<bool>(1);

    let _ = s.send(false);

    for num in 0..30 {
        let _ = send.send(Some(Message { num })).await;
    }

    for _ in 0..10 {
        let _ = send.send(None).await;
    }

    let mut set = JoinSet::new();
    for i in 0..10 {
        let recv_c = recv.clone();
        let r_c = s.subscribe();
        set.spawn(async move {
            let r = run(i, recv_c, r_c).await;
            format!("{}", r)
        });
    }

    set.spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                println!("CTRL+C received...");
                let _ = s.send(true);
                return "QUIT".to_string();
            }
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
                // we also shut down in case of error
            }
        };
        "GOT: CTRL+C".to_string()
    });

    while let Some(res) = set.join_next().await {
        let out = res;
        println!("GOT {:?}", out);
    }

    //
    // Wait for the tasks to finish.
    //
    // We drop our sender first because the recv() call otherwise
    // sleeps forever.
    drop(send);

    // When every sender has gone out of scope, the recv call
    // will return with an error. We ignore the error.
    let _ = recv.recv().await;
}
