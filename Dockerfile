# Use the official Rust image as a parent image
FROM rust:1.75-slim as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory in the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached if Cargo.toml doesn't change)
RUN cargo build --release && rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Create a new stage for the runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    iputils-ping \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/latencee /usr/local/bin/latencee

# Set the binary as the entrypoint
ENTRYPOINT ["latencee"]