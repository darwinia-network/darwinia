##
# Builder
##

FROM rust:1 as builder

ARG RUST_TOOLCHAIN=nightly-2020-10-06
ENV CARGO_TERM_COLOR=always

RUN apt-get update && apt-get install -y --no-install-recommends \
        clang \
        libclang-dev \
        cmake \
    && rm -rf /var/lib/apt/lists/*

RUN rustup default ${RUST_TOOLCHAIN} \
    && rustup target add wasm32-unknown-unknown

WORKDIR /src
COPY . .

RUN cargo build --release

##
# Final stage
##

FROM debian:stable-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/darwinia /usr/local/bin/

EXPOSE 30333 9933 9944

ENTRYPOINT [ "/usr/local/bin/darwinia" ]
