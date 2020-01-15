FROM ubuntu:bionic

# Fix locale and setup TERM
ENV LC_ALL=C.UTF-8 \
    TERM=xterm-256color

# Install dependencies
RUN apt-get update && \
    apt-get -y --no-install-recommends install apt-file && \
    apt-file update

RUN apt-get install -y --no-install-recommends \
        software-properties-common \
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
        libxml2-dev

# Install Java
RUN add-apt-repository ppa:openjdk-r/ppa && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        openjdk-11-jdk \
        ca-certificates-java

# Setup macOS cross-compile environment
COPY osxcross_setup.sh .
RUN ./osxcross_setup.sh && \
    rm osxcross_setup.sh

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --target x86_64-apple-darwin --profile minimal

# Setup path
ENV PATH="/root/.cargo/bin:/osxcross/bin:$PATH" \
    JAVA_HOME=/usr/lib/jvm/java-11-openjdk-amd64
