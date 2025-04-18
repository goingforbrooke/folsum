# # Instructions: Building/Testing on MacOS arm64

# (run all commands from repo root)
# (assumes that you've already run the cross build command associated with a given architecture).

## Build for x86_64

# Fill Dockerfile template
# tera --file xtask/docker/package_deb.template.Dockerfile --toml xtask/docker/target_x86_64.toml > xtask/docker/package_deb_x86_64.Dockerfile

# Build with:
# docker build -t folsum_deb_builder_x86_64 -f xtask/docker/package_deb_x86_64.Dockerfile .

# Do an ephemeral run and copy out the image with:
# docker run --rm -v $(pwd):/host_output folsum_deb_builder_x86_64

# Expect to see the .deb.
# ex. ls -> folsum_2.0.3-1_amd64.deb ...

## Build for aarch64

# Fill Dockerfile template
# tera --file xtask/docker/package_deb.template.Dockerfile --toml xtask/docker/target_aarch64.toml > xtask/docker/package_deb_aarch64.Dockerfile

# Build with:
# docker build -t folsum_deb_builder_aarch64 -f xtask/docker/package_deb_aarch64.Dockerfile .

# Do an ephemeral run and copy out the image with:
# docker run --rm -v $(pwd):/host_output folsum_deb_builder_aarch64

# Expect to see the .deb.
# ex. ls -> folsum_2.0.3-1_arm64.deb ...

# # Debugging

# SSH in with:
# docker run -it --rm folsum_deb_builder bash

# test installation:
# root@AAAAAAAAAAAA:/output# apt install ./folsum_2.0.3-1_amd64.deb
# Reading package lists... Done
# Building dependency tree... Done
# Reading state information... Done
# Note, selecting 'folsum' instead of './folsum_2.0.3-1_amd64.deb'
# The following NEW packages will be installed:
#   folsum
# 0 upgraded, 1 newly installed, 0 to remove and 0 not upgraded.
# Need to get 0 B/4394 kB of archives.
# After this operation, 18.0 MB of additional disk space will be used.
# Get:1 /output/folsum_2.0.3-1_amd64.deb folsum amd64 2.0.3-1 [4394 kB]
# debconf: delaying package configuration, since apt-utils is not installed
# Selecting previously unselected package folsum.
# (Reading database ... 4383 files and directories currently installed.)
# Preparing to unpack .../folsum_2.0.3-1_amd64.deb ...
#
# Progress: [  0%] [.......................................................................................................]
# Unpacking folsum (2.0.3-1) ...########...................................................................................]
#
# Setting up folsum (2.0.3-1) ...############################..............................................................]
#
# Progress: [ 60%] [#############################################################..........................................]

# test run with display *not* set:
#  root@AAAAAAAAAAAA:/output# folsum
# 21:41🧊logging.rs::folsum::loggingL173 Initialized logger with target file "/root/.local/share/folsum/logs/folsum.log"
# Error: WinitEventLoop(Os(OsError { line: 786, file: "/Users/flow/.cargo/registry/src/index.crates.io-6f17d22bba15001f/winit-0.29.15/src/platform_impl/linux/mod.rs", error: Misc("neither WAYLAND_DISPLAY nor WAYLAND_SOCKET nor DISPLAY is set.") }))

# test run with display set:

# 23:30🧊logging.rs::folsum::loggingL173 Initialized logger with target file "/root/.local/share/folsum/logs/folsum.log"
# Error: WinitEventLoop(NotSupported(NotSupportedError))
FROM ubuntu:25.04 AS deb_builder_{{ target_arch }}

# Install basic dependencies and curl (needed for rustup)
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    gcc \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

# Set path to use rust binaries (cargo, rustc, etc.)
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify installation (optional, for debug purposes)
RUN rustc --version && cargo --version

RUN cargo install cargo-deb

WORKDIR /usr/src/folsum

COPY target/{{ target_arch }}-unknown-linux-gnu/release/folsum target/{{ target_arch }}-unknown-linux-gnu/release/folsum
COPY Cargo.toml Cargo.toml

COPY folsum/Cargo.toml folsum/Cargo.toml
COPY folsum/src folsum/src

# Change README path to please `cargo deb`.
COPY README.md folsum/README.md

COPY xtask/Cargo.toml xtask/Cargo.toml
COPY xtask/src xtask/src

# And run:
RUN cargo deb -p folsum --no-strip --no-build --target {{ target_arch }}-unknown-linux-gnu

# Expect the deb package in /usr/src/folsum/target/<target_arch>-unknown-linux-gnu/debian/.
# ex. /usr/src/folsum/target/x86_64-unknown-linux-gnu/debian/folsum_2.0.3-1_amd64.deb \

# Translate target chip architecture to form the platform tuple.
FROM ubuntu:latest AS deb_extractor_{{ target_arch }}

VOLUME /output

COPY --from=deb_builder_{{ target_arch }} /usr/src/folsum/target/{{ target_arch }}-unknown-linux-gnu/debian/*.deb /output/

# Command to keep the container alive long enough for output
CMD ["bash", "-c", "cp /output/*.deb /host_output/ && echo 'Deb package copied!'"]
