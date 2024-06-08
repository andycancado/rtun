use std::sync::Arc;

use clap::Parser;
use futures::future::join_all;
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{mpsc, Mutex};

#[derive(Parser, Debug)]
#[command(
    name = "Rtun",
    version = "1.0",
    about = "A simple CLI for creating SSH tunnels."
)]
struct Args {
    #[arg(required = true, num_args=1.., help = "List of ports to tunnel")]
    ports: Vec<u16>,
    #[arg(required = true, long, help = "Host")]
    host: String,
}

async fn create_ssh_tunnel(
    local_port: u16,
    remote_port: u16,
    host: &str,
    shutdown: Arc<Mutex<mpsc::Receiver<()>>>,
) {
    let ssh_command = format!(
        "ssh -N -T -L {}:127.0.0.1:{} {}",
        local_port, remote_port, host
    );
    println!("Running: {}", ssh_command);
    let mut process = Command::new("sh")
        .arg("-c")
        .arg(&ssh_command)
        .spawn()
        .expect("Failed to spawn process");
    let mut x = shutdown.lock().await;
    tokio::select! {
        _ = x.recv() => {
            println!("Terminating SSH tunnel on port {}", local_port);
            let _ = process.kill().await;
        }
    }
}

async fn handle_signals() -> mpsc::Receiver<()> {
    let (sender, receiver) = mpsc::channel(1);
    let mut sigint =
        signal(SignalKind::interrupt()).expect("Failed to create SIGINT signal handler");
    let mut sigterm =
        signal(SignalKind::terminate()).expect("Failed to create SIGTERM signal handler");

    tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => {
                println!("Received SIGINT");
                let _ = sender.send(()).await;
            },
            _ = sigterm.recv() => {
                println!("Received SIGTERM");
                let _ = sender.send(()).await;
            }
        }
    });

    receiver
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let host = args.host;
    let shutdown_receiver = handle_signals().await;
    let s: Arc<Mutex<mpsc::Receiver<()>>> = Arc::new(Mutex::new(shutdown_receiver));
    let tunnel_tasks: Vec<_> = args
        .ports
        .iter()
        .map(|&port| {
            let shutdown_receiver = s.clone();
            create_ssh_tunnel(port, port, &host, shutdown_receiver)
        })
        .collect();

    let _ = join_all(tunnel_tasks).await;
}
