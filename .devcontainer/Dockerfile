FROM rust:latest

# Switch to nightly branch
RUN rustup default nightly

# Install standard library sources
RUN rustup component add rust-src

# Install QEMU to run and debug
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install qemu-system-x86
