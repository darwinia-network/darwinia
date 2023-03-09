FROM debian:stable-slim

RUN apt update && apt install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY dist/ /usr/local/bin

EXPOSE 9933 9944

ENTRYPOINT [ "/usr/local/bin/darwinia" ]
