FROM ubuntu:20.04

ARG USERNAME=darwinia
ARG USER_UID=1000
ARG USER_GID=$USER_UID

COPY dist/ /home/$USERNAME/darwinia-nodes/

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data \
    && chown -R $USERNAME:$USERNAME /home/$USERNAME \
    && chown -R $USERNAME:$USERNAME /data \
    && ln -s /home/$USERNAME/darwinia-nodes/darwinia /usr/local/bin/darwinia

USER $USERNAME
WORKDIR /home/$USERNAME/darwinia-nodes

EXPOSE 9933 9944

ENTRYPOINT [ "/usr/local/bin/darwinia" ]
