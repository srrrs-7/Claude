FROM rust:1.85 as builder

# Install mold linker dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    git \
    cmake \
    clang \
    build-essential \
    libstdc++-10-dev \
    libssl-dev \
    zlib1g-dev \
    pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Install mold linker from source
RUN git clone https://github.com/rui314/mold.git && \
    cd mold && \
    git checkout v2.4.0 && \
    mkdir build && \
    cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_CXX_COMPILER=clang++ .. && \
    cmake --build . -j $(nproc) && \
    cmake --install . && \
    cd ../.. && \
    rm -rf mold

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Configure cargo to use mold as the linker
RUN mkdir -p .cargo && \
    echo '[target.x86_64-unknown-linux-gnu]' > .cargo/config.toml && \
    echo 'rustflags = ["-C", "link-arg=-fuse-ld=mold"]' >> .cargo/config.toml && \
    echo 'linker = "clang"' >> .cargo/config.toml

# Create dummy source to build dependencies with mold
RUN mkdir src && \
    echo 'fn main() {println!("if you see this, the build broke")}' > src/main.rs && \
    echo 'pub fn init() {}' > src/lib.rs && \
    RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build --release && \
    rm -rf src

# Copy source code
COPY . .

# Build for release with mold linker
RUN RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build --release

# Second stage: final image
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/container-monitoring /app/container-monitoring

# Set entrypoint
ENTRYPOINT ["/app/container-monitoring"]
