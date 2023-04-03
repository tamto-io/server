FROM ubuntu:lunar AS builder

WORKDIR /usr/src/app

RUN apt update && apt install -y libssl-dev pkg-config protobuf-compiler curl gcc

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release

FROM builder as node-builder

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo install --path ./server

FROM builder as admin-builder

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo install --path ./bin/cli-admin

FROM ubuntu:lunar as admin

COPY --from=admin-builder /root/.cargo/bin/tamto-* /usr/bin/

FROM ubuntu:lunar as node

COPY --from=node-builder /root/.cargo/bin/server /usr/bin/server
COPY scripts/docker-entrypoint.sh /usr/bin/

CMD ["docker-entrypoint.sh"]
