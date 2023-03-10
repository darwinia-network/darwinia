FROM debian:stable-slim

ARG USERNAME=darwinia
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME && \
    useradd --uid $USER_UID --gid $USER_GID -m $USERNAME && \
    apt-get update && apt-get install -y --no-install-recommends sudo ca-certificates && \
    echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME && \
    chmod 0440 /etc/sudoers.d/$USERNAME && \
    rm -rf /var/lib/apt/lists/*

COPY dist/ /home/$USERNAME/darwinia-nodes/

RUN chown -R $USERNAME:$USERNAME /home/$USERNAME/darwinia-nodes && \
    ln -s /home/$USERNAME/darwinia-nodes/darwinia /usr/local/bin/darwinia

USER $USERNAME

EXPOSE 9933 9944

ENTRYPOINT [ "/usr/local/bin/darwinia" ]
