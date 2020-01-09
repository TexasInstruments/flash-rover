FROM ubuntu:bionic

# Fix locale and setup TERM
ENV LC_ALL=C.UTF-8 \
    TERM=xterm-256color

# Use root user in order to perform modifications to the image
USER root

# Install dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        wget \
        curl \
        clang \
        cmake \
        git \
        patch \
        python \
        libssl-dev \
        lzma-dev \
        libxml2-dev && \
    apt-get clean

# Setup developer user
RUN useradd -ms /bin/bash developer
USER developer
WORKDIR /home/developer

# Setup macOS cross-compile environment
COPY osxcross_setup.sh .
RUN ./osxcross_setup.sh && \
    rm osxcross_setup.sh

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --target x86_64-apple-darwin --profile minimal
ENV PATH="/home/developer/.cargo/bin:${PATH}"
