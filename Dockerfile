# Build stage
FROM rust:1.86 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/feedtape-backend /app/feedtape-backend

# Copy migrations for runtime
COPY --from=builder /app/migrations /app/migrations

# Expose port (Railway will override with PORT env var)
EXPOSE 8080

# Run the binary
CMD ["./feedtape-backend"]
