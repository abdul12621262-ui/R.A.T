# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY server/ ./server/
COPY apps/ ./apps/

# Build the signaling server
RUN cargo build --release -p rat-signaling

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/rat-signaling /app/rat-signaling

# Expose the signaling port
EXPOSE 4899

# Set environment variable for port
ENV PORT=4899

# Run the signaling server
CMD ["/app/rat-signaling"]
