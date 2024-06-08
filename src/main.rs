use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures::future::join_all;
use ratatui::{prelude::*, widgets::*};
use std::io::stdout;
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
    let mut rx = shutdown.lock().await;
    tokio::select! {
        _ = rx.recv() => {
            println!("Terminating SSH tunnel on port {}", local_port);
            let _ = process.kill().await;
        }
    }
}

async fn handle_signals(tx: Arc<mpsc::Sender<()>>) {
    let mut sigint =
        signal(SignalKind::interrupt()).expect("Failed to create SIGINT signal handler");
    let mut sigterm =
        signal(SignalKind::terminate()).expect("Failed to create SIGTERM signal handler");

    tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => {
                println!("Received SIGINT");
                let _ = tx.send(()).await;
            },
            _ = sigterm.recv() => {
                println!("Received SIGTERM");
                let _ = tx.send(()).await;
            }
        }
    });
}

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let args = Args::try_parse().map_err(|err| {
        stdout().execute(LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        println!("{}", err);
        std::process::exit(1);
    });
    let args = args.unwrap();
    let host = args.host;
    let (tx, rx) = mpsc::channel(1);
    let sender = Arc::new(tx);
    handle_signals(sender.clone()).await;
    let shutdown_receiver = Arc::new(Mutex::new(rx));
    let tunnel_tasks: Vec<_> = args
        .ports
        .iter()
        .map(|&port| {
            let shutdown_receiver = shutdown_receiver.clone();
            create_ssh_tunnel(port, port, &host, shutdown_receiver)
        })
        .collect();
    let ports = args
        .ports
        .iter()
        .map(|&s| s.to_string())
        .collect::<Vec<String>>();
    let tasks = join_all(tunnel_tasks);
    loop {
        let _ = terminal.draw(|frame| {
            let area = frame.size();
            let items = &ports;
            let items = items.iter().map(|t| format!("PORTS >>>> {}", t));
            let list = List::new(items)
                .block(Block::bordered().title("Rtun - SSH Tunnel Manager (hit q to quit)"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);

            frame.render_widget(list, centered_rect(area, 50, 50));
        });
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    stdout().execute(LeaveAlternateScreen)?;
                    disable_raw_mode()?;
                    // tasks.await;
                    // std::mem::drop(tasks);
                    break;
                }
            }
        }
    }

    let _ = sender.send(()).await;
    tasks.await;
    Ok(())
}
