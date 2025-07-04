use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use smol::{channel, Timer};
use std::{
    collections::VecDeque,
    io::{self, Write},
    process::Command,
    time::{Duration, Instant},
};

const GRAPH_WIDTH: usize = 60;
const GRAPH_HISTORY_MINUTES: usize = 10;

#[derive(Clone)]
pub struct ServerStatus {
    pub name: String,
    pub latency: Option<Duration>,
    pub last_update: Instant,
    pub status: ConnectionStatus,
    pub history: VecDeque<(Instant, ConnectionStatus)>,
}

#[derive(Clone, PartialEq)]
pub enum ConnectionStatus {
    Good,    // < 50ms
    Fair,    // 50-150ms
    Poor,    // 150-500ms
    Timeout, // > 500ms or failed
}

impl ConnectionStatus {
    fn color(&self) -> Color {
        match self {
            ConnectionStatus::Good => Color::Green,
            ConnectionStatus::Fair => Color::Yellow,
            ConnectionStatus::Poor => Color::Red,
            ConnectionStatus::Timeout => Color::DarkRed,
        }
    }

    fn symbol(&self) -> &str {
        match self {
            ConnectionStatus::Good => "●",
            ConnectionStatus::Fair => "◐",
            ConnectionStatus::Poor => "◑",
            ConnectionStatus::Timeout => "○",
        }
    }
}

pub fn ping_host(host: &str) -> Option<Duration> {
    let start = Instant::now();
    
    // Simple ping using system ping command
    let output = Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("-W")
        .arg("1000") // 1 second timeout
        .arg(host)
        .output()
        .ok()?;

    if output.status.success() {
        Some(start.elapsed())
    } else {
        None
    }
}

pub fn classify_latency(latency: Option<Duration>) -> ConnectionStatus {
    match latency {
        Some(lat) if lat < Duration::from_millis(50) => ConnectionStatus::Good,
        Some(lat) if lat < Duration::from_millis(150) => ConnectionStatus::Fair,
        Some(lat) if lat < Duration::from_millis(500) => ConnectionStatus::Poor,
        _ => ConnectionStatus::Timeout,
    }
}

async fn monitor_server(name: String, host: String, sender: channel::Sender<ServerStatus>) {
    let mut history = VecDeque::new();
    
    loop {
        let latency = ping_host(&host);
        let status = classify_latency(latency);
        let now = Instant::now();

        // Add to history
        history.push_back((now, status.clone()));
        
        // Keep only last N minutes of history
        let cutoff = now - Duration::from_secs(GRAPH_HISTORY_MINUTES as u64 * 60);
        while let Some((timestamp, _)) = history.front() {
            if *timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }

        let server_status = ServerStatus {
            name: name.clone(),
            latency,
            last_update: now,
            status,
            history: history.clone(),
        };

        if sender.send(server_status).await.is_err() {
            break;
        }

        Timer::after(Duration::from_secs(2)).await;
    }
}

fn draw_graph(history: &VecDeque<(Instant, ConnectionStatus)>) -> String {
    if history.is_empty() {
        return " ".repeat(GRAPH_WIDTH);
    }

    let now = Instant::now();
    let start_time = now - Duration::from_secs(GRAPH_HISTORY_MINUTES as u64 * 60);
    let time_per_char = Duration::from_secs(GRAPH_HISTORY_MINUTES as u64 * 60) / GRAPH_WIDTH as u32;
    
    let mut graph = vec![' '; GRAPH_WIDTH];
    
    for (timestamp, status) in history {
        if *timestamp >= start_time {
            let elapsed = timestamp.duration_since(start_time);
            let pos = (elapsed.as_secs_f64() / time_per_char.as_secs_f64()) as usize;
            if pos < GRAPH_WIDTH {
                graph[pos] = match status {
                    ConnectionStatus::Good => '●',
                    ConnectionStatus::Fair => '◐',
                    ConnectionStatus::Poor => '◑',
                    ConnectionStatus::Timeout => '○',
                };
            }
        }
    }
    
    graph.into_iter().collect()
}

fn draw_ui(servers: &[ServerStatus]) -> io::Result<()> {
    execute!(
        io::stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    println!("🌐 Latencee - Network Latency Monitor");
    println!("Press 'q' to quit\n");

    for (i, server) in servers.iter().enumerate() {
        let row = (i * 3 + 3) as u16;
        execute!(io::stdout(), cursor::MoveTo(0, row))?;
        
        // Server name and current status
        execute!(io::stdout(), SetForegroundColor(server.status.color()))?;
        print!("{} ", server.status.symbol());
        execute!(io::stdout(), ResetColor)?;
        
        print!("{:<20}", server.name);
        
        match server.latency {
            Some(lat) => {
                execute!(io::stdout(), SetForegroundColor(server.status.color()))?;
                print!("{:>8.0}ms", lat.as_millis());
                execute!(io::stdout(), ResetColor)?;
            }
            None => {
                execute!(io::stdout(), SetForegroundColor(Color::DarkRed))?;
                print!("{:>8}", "TIMEOUT");
                execute!(io::stdout(), ResetColor)?;
            }
        }
        
        let age = server.last_update.elapsed().as_secs();
        if age > 5 {
            execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
            print!(" ({}s ago)", age);
            execute!(io::stdout(), ResetColor)?;
        }
        
        println!();
        
        // Graph line
        execute!(io::stdout(), cursor::MoveTo(2, row + 1))?;
        let graph = draw_graph(&server.history);
        
        // Draw graph with colors
        for ch in graph.chars() {
            if ch != ' ' {
                let color = match ch {
                    '●' => Color::Green,
                    '◐' => Color::Yellow,
                    '◑' => Color::Red,
                    '○' => Color::DarkRed,
                    _ => Color::White,
                };
                execute!(io::stdout(), SetForegroundColor(color))?;
                print!("{}", ch);
                execute!(io::stdout(), ResetColor)?;
            } else {
                print!("·");
            }
        }
        
        println!(" [{} min]", GRAPH_HISTORY_MINUTES);
    }

    let legend_row = (servers.len() * 3 + 5) as u16;
    execute!(io::stdout(), cursor::MoveTo(0, legend_row))?;
    println!("Legend:");
    execute!(io::stdout(), SetForegroundColor(Color::Green))?;
    print!("● Good (<50ms)  ");
    execute!(io::stdout(), SetForegroundColor(Color::Yellow))?;
    print!("◐ Fair (50-150ms)  ");
    execute!(io::stdout(), SetForegroundColor(Color::Red))?;
    print!("◑ Poor (150-500ms)  ");
    execute!(io::stdout(), SetForegroundColor(Color::DarkRed))?;
    print!("○ Timeout (>500ms)");
    execute!(io::stdout(), ResetColor)?;
    
    io::stdout().flush()?;
    Ok(())
}

pub fn get_default_servers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Google DNS", "8.8.8.8"),
        ("Cloudflare DNS", "1.1.1.1"),
        ("Google", "google.com"),
        ("GitHub", "github.com"),
        ("Stack Overflow", "stackoverflow.com"),
    ]
}

fn main() -> io::Result<()> {
    let servers = get_default_servers();

    smol::block_on(async {
        terminal::enable_raw_mode()?;
        
        let (sender, receiver) = channel::unbounded::<ServerStatus>();
        let mut server_statuses = Vec::new();

        // Initialize server statuses
        for (name, _host) in &servers {
            server_statuses.push(ServerStatus {
                name: name.to_string(),
                latency: None,
                last_update: Instant::now(),
                status: ConnectionStatus::Timeout,
                history: VecDeque::new(),
            });
        }

        // Start monitoring tasks
        for (name, host) in servers {
            let sender = sender.clone();
            smol::spawn(monitor_server(name.to_string(), host.to_string(), sender)).detach();
        }

        // Initial draw
        draw_ui(&server_statuses)?;

        loop {
            // Check for keyboard input
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }

            // Update server statuses
            while let Ok(status) = receiver.try_recv() {
                if let Some(server) = server_statuses.iter_mut().find(|s| s.name == status.name) {
                    *server = status;
                }
            }

            // Redraw UI
            draw_ui(&server_statuses)?;
            
            Timer::after(Duration::from_millis(500)).await;
        }

        terminal::disable_raw_mode()?;
        execute!(io::stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        println!("Goodbye!");
        
        Ok(())
    })
}