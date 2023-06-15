mod cli;

use crate::cli::Args;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::process::exit;
use std::time::Duration;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};

async fn send_ctrl_c(child: &mut tokio::process::Child) {
    let timeout_duration = Duration::from_secs(5);

    let ctrl_c_signal =
        signal(SignalKind::interrupt()).expect("Failed to register Ctrl+C handler.");
    tokio::pin!(ctrl_c_signal);

    tokio::select! {
        _ = tokio::time::sleep(timeout_duration) => {
            // Timeout reached
            println!("Timeout reached. Sending SIGTERM to the child process.");
            signal::kill(Pid::from_raw(child.id().unwrap().try_into().unwrap()), Signal::SIGINT).unwrap();

        }
        _ = ctrl_c_signal.recv() => {
            // Ctrl+C signal received
            println!("Ctrl+C signal received. Sending SIGINT to the child process.");
            signal::kill(Pid::from_raw(child.id().unwrap().try_into().unwrap()), Signal::SIGINT).unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::new();

    println!("Execute command: {:?}", args.execute);

    // Example command and arguments
    let split_execute: Vec<&str> = args.execute.split(' ').collect();
    let exec_command = split_execute[0];
    let exec_args = &split_execute[1..];

    // Spawn sub-process
    let mut child = Command::new(exec_command)
        .args(exec_args)
        .spawn()
        .expect("Failed to spawn child process.");

    // Send Ctrl+C signal
    send_ctrl_c(&mut child).await;

    // Wait for the child process to finish
    let status = child
        .wait()
        .await
        .expect("Failed to wait for child process.");

    if !status.success() {
        println!("Child process exited with an error.");
        exit(1);
    }
}
