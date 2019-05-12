#!/bin/bash

docker run --rm --entrypoint /bin/bash \
    -w '/project' \
    -v "$(pwd):/project" \
    ubuntu:18.04 -c \
    'apt-get update \
    && apt-get -y install curl build-essential \
    && curl --proto "=https" --tlsv1.2 -sSf -o /tmp/rustup-init https://sh.rustup.rs \
    && bash /tmp/rustup-init -y \
    && source ~/.cargo/env \
    && cargo clean \
    && cargo build --release'
