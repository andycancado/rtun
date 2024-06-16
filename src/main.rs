use std::sync::Arc;

use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};
use ssh2_config::{ParseRule, SshConfig};
use std::io::stdout;
use std::io::BufReader;
use std::{env, fs::File};
use tokio::process::Command;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::{mpsc, Mutex};
use tui_textarea::TextArea;

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

const CONFIG_PATH: &str = ".ssh/config";

fn get_hosts() -> Vec<String> {
    let home_dir = env::home_dir().unwrap();
    let mut reader = BufReader::new(
        File::open(home_dir.join(CONFIG_PATH)).expect("Could not open configuration file"),
    );
    let config = SshConfig::default()
        .parse(&mut reader, ParseRule::STRICT)
        .expect("Failed to parse configuration");
    let hosts = config.get_hosts();
    let hosts: Vec<_> = hosts
        .iter()
        .filter_map(|h| match h.pattern.first() {
            Some(host_clause) => {
                if host_clause.pattern == "*" {
                    None
                } else {
                    Some(host_clause.pattern.clone())
                }
            }
            _ => None,
        })
        .collect();
    hosts
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

async fn handle_signals(tx: Arc<Mutex<mpsc::Sender<()>>>) {
    let mut sigint =
        signal(SignalKind::interrupt()).expect("Failed to create SIGINT signal handler");
    let mut sigterm =
        signal(SignalKind::terminate()).expect("Failed to create SIGTERM signal handler");

    tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => {
                println!("Received SIGINT");
                let _ = tx.lock().await.send(()).await;
            },
            _ = sigterm.recv() => {
                println!("Received SIGTERM");
                let _ = tx.lock().await.send(()).await;
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

fn get_text_area<'a>() -> TextArea<'a> {
    let mut textarea = TextArea::default();
    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .title("Hit enter to set port"),
    );
    textarea.set_style(Style::default().fg(Color::Yellow));
    textarea.set_placeholder_style(Style::default());
    textarea.set_placeholder_text("Host_name 1234:45321");
    textarea
}

fn get_config_from_str(input: &str) -> Result<(String, u16, u16), &'static str> {
    let parts: Vec<&str> = input.split(' ').collect();
    if parts.len() != 2 {
        return Err("Input does not match expected format 'HOST_NAME 12234:45321'");
    }

    let host_name = parts[0].to_string();

    let ports: Vec<&str> = parts[1].split(':').collect();
    if ports.len() != 2 {
        return Err("Ports part does not match expected format '12234:45321'");
    }

    let host_port = ports[0]
        .parse::<u16>()
        .map_err(|_| "Failed to parse host_port")?;
    let remote_port = ports[1]
        .parse::<u16>()
        .map_err(|_| "Failed to parse remote_port")?;

    Ok((host_name, host_port, remote_port))
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    let (tx, rx) = mpsc::channel(1);
    let sender = Arc::new(Mutex::new(tx));
    let shutdown_receiver = Arc::new(Mutex::new(rx));
    handle_signals(sender.clone()).await;

    let mut ports: Vec<String> = Vec::new();
    let mut textarea = get_text_area();
    let mut new_port: Option<String> = None;
    loop {
        let _ = terminal.draw(|frame| {
            let area = frame.size();
            let items = &ports;
            let items: Vec<String> = items.iter().map(|t| format!("{}:{}", t, t)).collect();

            let list = List::new(items)
                .block(
                    Block::bordered()
                        .title("Rtun - SSH Tunnel Manager (hit esc to quit, n to new tunnel)"),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);
            let center = centered_rect(area, 50, 50);
            frame.render_widget(list, center);

            let list_hosts = List::new(get_hosts())
                .style(Style::default().fg(Color::White))
                .direction(ListDirection::TopToBottom);

            frame.render_widget(
                list_hosts,
                Rect::new(
                    center.x + (center.width / 2),
                    center.y + 1,
                    center.width / 2,
                    center.height,
                ),
            );

            if new_port.is_some() {
                let new_area = Rect::new(center.x, center.y + center.height, center.width, 20);
                frame.render_widget(textarea.widget(), centered_rect(new_area, 100, 100));
            }
        });
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            if new_port.is_none() {
                                stdout().execute(LeaveAlternateScreen)?;
                                disable_raw_mode()?;
                                break;
                            } else {
                                new_port = None;
                            }
                        }
                        KeyCode::Char('n') if new_port.is_none() => {
                            new_port = Some("".to_string());
                            textarea = get_text_area();
                        }
                        KeyCode::Backspace => {
                            textarea.delete_char();
                        }
                        KeyCode::Char(_) if new_port.is_some() => {
                            textarea.input(key);
                            new_port = match textarea.lines().first() {
                                Some(l) => Some(l.to_string()),
                                _ => Some("".to_string()),
                            };
                        }
                        KeyCode::Enter if new_port.is_some() => {
                            if let Some(ref l) = &new_port {
                                match get_config_from_str(l) {
                                    Ok((host_name, host_port, remote_port)) => {
                                        let shutdown_receiver = shutdown_receiver.clone();
                                        let _jh = tokio::spawn(async move {
                                            create_ssh_tunnel(
                                                host_port,
                                                remote_port,
                                                host_name.as_str(),
                                                shutdown_receiver.clone(),
                                            )
                                            .await;
                                        });
                                        ports.push(host_port.to_string());
                                    }
                                    Err(e) => {
                                        println!("Error: {}", e);
                                    }
                                }
                            }
                            new_port = None;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    for _ in ports.iter() {
        let _ = sender.lock().await.send(()).await;
    }
    Ok(())
}
