mod cli;
mod contents;

use crate::cli::Args;

use colored::Colorize;
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
use std::{thread, time::Duration};
use tokio::process::Command;
use tokio::spawn;

use crate::contents::enums;

static CHILD_ID: AtomicI32 = AtomicI32::new(0);

pub fn set_child_id(val: i32) {
    CHILD_ID.store(val, Ordering::SeqCst)
}

pub fn init_child_id() {
    set_child_id(0)
}

pub fn get_child_id() -> i32 {
    CHILD_ID.load(Ordering::SeqCst)
}

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

async fn async_watch<P: AsRef<Path>>(
    path: P,
    file_event: enums::FileEvent,
    signal: Signal,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                println!(
                    "{} {}{:?}, {}{:?}",
                    "[howl]".bright_magenta().bold(),
                    "Got event: ".yellow(),
                    event.kind,
                    "monitor: ".yellow(),
                    file_event
                );

                let need_send_signal: bool = file_event == enums::FileEvent::Any
                    || (event.kind.is_access() && file_event == enums::FileEvent::Access)
                    || (event.kind.is_create() && file_event == enums::FileEvent::Create)
                    || (event.kind.is_modify() && file_event == enums::FileEvent::Modify)
                    || (event.kind.is_remove() && file_event == enums::FileEvent::Remove);

                let child_id: i32 = get_child_id();
                if need_send_signal && child_id > 0 {
                    println!(
                        "{} {}{}",
                        "[howl]".bright_magenta().bold(),
                        "Send signal to child: ".bright_blue(),
                        child_id
                    );
                    signal::kill(Pid::from_raw(child_id), signal).unwrap();
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

    println!(
        r#"
     __  __     ______     __     __     __
    /\ \_\ \   /\  __ \   /\ \  _ \ \   /\ \
    \ \  __ \  \ \ \/\ \  \ \ \/ ".\ \  \ \ \____
     \ \_\ \_\  \ \_____\  \ \__/".~\_\  \ \_____\
      \/_/\/_/   \/_____/   \/_/   \/_/   \/_____/
    "#
    );

    println!(
        "{} {}{:?}",
        "[howl]".bright_magenta().bold(),
        "Execute command: ".blue(),
        args.execute
    );
    println!(
        "{} {}{:?}",
        "[howl]".bright_magenta().bold(),
        "Monitor path: ".blue(),
        args.path
    );

    // Example command and arguments
    let split_execute: Vec<&str> = args.execute.split(' ').collect();
    let exec_command = split_execute[0];
    let exec_args = &split_execute[1..];

    let mut child_container: Option<tokio::process::Child>;

    spawn(async move {
        if let Err(e) = async_watch(args.path.as_path(), args.file_event, args.signal).await {
            println!("{}{:?}", "error: ".red(), e)
        }
    });

    loop {
        println!(
            "{} {}",
            "[howl]".bright_magenta().bold(),
            "----- execute -----".green()
        );
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
            println!(
                "{} {}",
                "[howl]".bright_magenta().bold(),
                "Child process exited with an error.".red()
            );
            exit(1);
        }
        thread::sleep(Duration::from_millis(100));
    }
}
