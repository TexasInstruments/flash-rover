FROM rustembedded/cross:x86_64-unknown-linux-gnu

# Install Java
RUN apt-get clean && \
    apt-get update --fix-missing && \
    apt-get install -y --no-install-recommends \
        software-properties-common && \
    add-apt-repository -y ppa:openjdk-r/ppa && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        openjdk-11-jdk

# Set necessary environment variables
ENV JAVA_HOME=/usr/lib/jvm/java-11-openjdk-amd64

