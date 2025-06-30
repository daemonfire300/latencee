use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    style::{Color, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use smol::{channel, Timer};
use std::{
    io::{self, Write},
    process::Command,
    time::{Duration, Instant},
};

#[derive(Clone)]
struct ServerStatus {
    name: String,
    latency: Option<Duration>,
    last_update: Instant,
    status: ConnectionStatus,
}

#[derive(Clone, PartialEq)]
enum ConnectionStatus {
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

fn ping_host(host: &str) -> Option<Duration> {
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

async fn monitor_server(name: String, host: String, sender: channel::Sender<ServerStatus>) {
    loop {
        let latency = ping_host(&host);
        let status = match latency {
            Some(lat) if lat < Duration::from_millis(50) => ConnectionStatus::Good,
            Some(lat) if lat < Duration::from_millis(150) => ConnectionStatus::Fair,
            Some(lat) if lat < Duration::from_millis(500) => ConnectionStatus::Poor,
            _ => ConnectionStatus::Timeout,
        };

        let server_status = ServerStatus {
            name: name.clone(),
            latency,
            last_update: Instant::now(),
            status,
        };

        if sender.send(server_status).await.is_err() {
            break;
        }

        Timer::after(Duration::from_secs(2)).await;
    }
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
        execute!(io::stdout(), cursor::MoveTo(0, (i + 3) as u16))?;
        
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
    }

    println!("\nLegend:");
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

fn main() -> io::Result<()> {
    let servers = vec![
        ("Google DNS", "8.8.8.8"),
        ("Cloudflare DNS", "1.1.1.1"),
        ("Google", "google.com"),
        ("GitHub", "github.com"),
        ("Stack Overflow", "stackoverflow.com"),
    ];

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