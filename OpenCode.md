# Latencee - Network Latency Monitor

A terminal TUI application that measures and visualizes network latency to multiple servers with colorful indicators and historical graphs.

## Current Status

✅ **Completed Features:**
- Real-time latency monitoring to multiple servers (Google DNS, Cloudflare DNS, Google, GitHub, Stack Overflow)
- Colorful status indicators with symbols:
  - ● Green: Good latency (<50ms)
  - ◐ Yellow: Fair latency (50-150ms) 
  - ◑ Red: Poor latency (150-500ms)
  - ○ Dark Red: Timeout (>500ms or failed)
- Historical graphs showing last 10 minutes of latency data using Unicode dots
- Async monitoring using `smol` runtime (lightweight alternative to tokio)
- Cross-platform ping using system `ping` command
- Clean TUI with keyboard controls (press 'q' to quit)
- Minimal dependencies (only `crossterm` and `smol`)

## Architecture

- **Runtime**: Uses `smol` async runtime for lightweight concurrency
- **UI**: Terminal-based using `crossterm` for cross-platform terminal control
- **Networking**: System `ping` command for reliable cross-platform latency measurement
- **Data Structure**: Each server maintains a history queue of the last 10 minutes of measurements
- **Graph Rendering**: 60-character wide graphs using Unicode dot characters with appropriate colors

## Commands

```bash
# Build and run
cargo run

# Check for errors
cargo check

# Build release version
cargo build --release
```

## Project Goals

- ✅ Measure latency/speed to multiple servers
- ✅ Colorful indicators for connection quality
- ✅ Historical visualization with graphs
- ✅ Minimal dependencies
- ✅ Cross-platform (macOS/Linux)
- ✅ Uses smol instead of tokio for efficiency
- 🔄 Docker support for CI/testing (Dockerfile exists but needs testing)

## Usage

Run the application to monitor network latency in real-time. The TUI shows:
1. Current latency and status for each server
2. Historical graph of the last 10 minutes
3. Legend explaining the color coding
4. Age indicator if data is stale (>5 seconds)

Perfect for monitoring network stability on plane/train WiFi or any unstable connection.
