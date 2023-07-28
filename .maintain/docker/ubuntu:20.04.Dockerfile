FROM ubuntu:20.04

RUN apt-get update \
    && apt install --no-install-recommends -y \
        ca-certificates curl git python3-pip \
        clang make gcc g++ libssl-dev pkg-config protobuf-compiler \
    && pip3 install --upgrade pip \
    && pip3 install cmake --upgrade \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none \
    && git config --global --add safe.directory /build

ENV PATH="$PATH:/root/.cargo/bin"

WORKDIR /build
