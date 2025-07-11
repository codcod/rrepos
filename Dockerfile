# Build stage
FROM rust:1.88-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:12

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false rrepos

# Copy the binary from builder stage
COPY --from=builder /app/target/release/rrepos /usr/local/bin/rrepos

# Set the user
USER rrepos

# Set the working directory
WORKDIR /workspace

# Set the entrypoint
ENTRYPOINT ["rrepos"]
CMD ["--help"]
