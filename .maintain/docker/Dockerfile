##
# Only for CI using, for building process please refer to .github/workflows.
##

FROM ubuntu:focal

ARG CI_GIT_TAG
ARG CI_GIT_SHA
ARG CI_BUILD_AT

# See:
# https://github.com/opencontainers/image-spec/blob/main/annotations.md
# https://github.com/paritytech/polkadot/blob/master/scripts/dockerfiles/polkadot_injected_release.Dockerfile
LABEL network.darwinia.image.created="${CI_BUILD_AT}" \
	network.darwinia.image.authors="hello@darwinia.network" \
	network.darwinia.image.url="https://github.com/darwinia-network/dariwnia" \
	network.darwinia.image.documentation="https://docs.darwinia.network/" \
	network.darwinia.image.source="https://github.com/darwinia-network/darwinia/blob/master/.maintain/docker/Dockerfile" \
	network.darwinia.image.version="${CI_GIT_TAG}" \
	network.darwinia.image.revision="${CI_GIT_SHA}" \
	network.darwinia.image.licenses="GPL-3.0" \
	network.darwinia.image.title="Darwinia" \
	network.darwinia.image.description="Implementation of a https://darwinia.network node in Rust based on the Substrate framework."

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY shared/darwinia /usr/local/bin/darwinia
ENTRYPOINT [ "/usr/local/bin/darwinia" ]
