mod cli;

use crate::cli::Args;

use core::sync::atomic::{AtomicI32, Ordering};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process::exit;
use tokio::process::Command;
// use tokio::signal::unix::{signal, SignalKind};
use tokio::spawn;

// #[derive(PartialEq)]
// enum ChildKillResult {
//     Manual,
// }

static CHILD_ID: AtomicI32 = AtomicI32::new(0);

pub fn set_child_id(val: i32) {
    CHILD_ID.store(val, Ordering::Relaxed)
}

pub fn init_child_id() {
    set_child_id(0)
}

pub fn get_child_id() -> i32 {
    CHILD_ID.load(Ordering::Relaxed)
}

// async fn send_ctrl_c() -> ChildKillResult {
//     let ctrl_c_signal =
//         signal(SignalKind::interrupt()).expect("Failed to register Ctrl+C handler.");
//     tokio::pin!(ctrl_c_signal);

//     tokio::select! {
//         _ = ctrl_c_signal.recv() => {
//             // Ctrl+C signal received
//             println!("Ctrl+C signal received. Sending SIGINT to the child process.");
//             let child_id: i32 = get_child_id();
//             if child_id > 0 {
//                 signal::kill(Pid::from_raw(child_id), Signal::SIGINT).unwrap();
//             }
//             ChildKillResult::Manual
//         }
//     }
// }

fn keep_running_command(command: String, args: Vec<&str>) -> Option<tokio::process::Child> {
    // Spawn sub-process
    let child = Command::new(command)
        .args(args)
        .spawn()
        .expect("Failed to spawn child process.");

    Some(child)
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                println!("changed: {:?}", event);

                let child_id: i32 = get_child_id();
                println!("changed to get child_id: {}", child_id);
                if child_id > 0 {
                    println!("Send SIGINT to get child_id: {}", child_id);
                    signal::kill(Pid::from_raw(child_id), Signal::SIGINT).unwrap();
                    init_child_id()
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::new();

    println!("Execute command: {:?}", args.execute);

    // Example command and arguments
    let split_execute: Vec<&str> = args.execute.split(' ').collect();
    let exec_command = split_execute[0];
    let exec_args = &split_execute[1..];

    let mut child_container: Option<tokio::process::Child>;

    spawn(async move {
        if let Err(e) = async_watch("./src").await {
            println!("error: {:?}", e)
        }
    });

    loop {
        init_child_id();
        child_container = keep_running_command(exec_command.to_string(), exec_args.to_vec());

        if child_container.is_none() {
            break;
        }
        let mut child = child_container.unwrap();
        set_child_id(child.id().unwrap().try_into().unwrap());

        // Wait for the child process to finish
        let status = child
            .wait()
            .await
            .expect("Failed to wait for child process.");

        if !status.success() {
            println!("Child process exited with an error.");
            exit(1);
        }
        // let res = send_ctrl_c().await;

        // if res == ChildKillResult::Manual {
        //     println!("Child process exited by manual.");
        //     break;
        // }
    }
}
