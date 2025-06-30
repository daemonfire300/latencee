# Latencee - Network Latency Monitor

A terminal TUI application that measures and visualizes network latency to multiple servers with colorful indicators.

## Features

- Real-time latency monitoring to multiple servers
- Colorful status indicators:
  - 🟢 Good (< 50ms)
  - 🟡 Fair (50-150ms) 
  - 🔴 Poor (150-500ms)
  - ⚫ Timeout (> 500ms or failed)
- Minimal dependencies using `smol` async runtime
- Cross-platform support (macOS and Linux)

## Usage

### Running locally

```bash
cargo run
```

### Building release version

```bash
cargo build --release
./target/release/latencee
```

### Using Docker

```bash
# Build the Docker image
docker build -t latencee .

# Run the container
docker run -it --rm latencee
```

## Controls

- Press `q` to quit the application

## Monitored Servers

The application monitors latency to:
- Google DNS (8.8.8.8)
- Cloudflare DNS (1.1.1.1)
- Google (google.com)
- GitHub (github.com)
- Stack Overflow (stackoverflow.com)

## Requirements

- Rust 1.75 or later
- System `ping` command available
- Terminal with color support

## Dependencies

- `crossterm` - Cross-platform terminal manipulation
- `smol` - Lightweight async runtime

## Architecture

The application uses:
- `smol` async runtime for lightweight concurrency
- System `ping` command for latency measurement
- `crossterm` for terminal UI and color output
- Minimal external dependencies as requested