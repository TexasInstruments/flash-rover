FROM artifactory.itg.ti.com/docker-epd-cmcu-local/ubuntu-developer:bionic-20190912.1-0

# Fix locale, and setup TERM and conan required variables
ENV LC_ALL=C.UTF-8 \
    TERM=xterm-256color \
    CONAN_ENABLED=true

# Use root user in order to perform modifications to the image.
USER root

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        cmake \
        clang \
        gcc \
        g++ \
        mingw-w64 \
        zlib1g-dev \
        libmpc-dev \
        libmpfr-dev \
        libgmp-dev \
        libxml2-dev && \
    apt-get clean

USER developer

COPY osxcross_setup.sh .
RUN ./osxcross_setup.sh

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN ~/.cargo/bin/rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-apple-darwin \
    x86_64-pc-windows-gnu

