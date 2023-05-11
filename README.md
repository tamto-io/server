# Chord

Chord protocol implementation in Rust.

![Build Status](https://github.com/tamto-labs/server/actions/workflows/rust.yaml/badge.svg)

This repo contains a Rust implementation of the Chord protocol.

> **Warning** 
>
> This is a work in progress and is not yet ready for production use.

## Current Status

- [x] Basic Chord protocol
- [x] gRPC API for between nodes communication
- [x] Node leaving the ring. Right now if a node leaves the ring, the ring will be broken.
- [ ] Data storage

## Usage

### Build

```bash
make build
```

### Run

```bash
make run
```

If you want to run the node with different configuration, you can use the following command:

```bash
cargo run -p server -- --help
```

You can also run multiple nodes at the same time:

```bash
make run-local
```

It will run 10 nodes on the following ports `50050` to `50060`. You can find logs from all nodes in `nohup.out` file.

### CLI

There is also a CLI tool which can be used to interact with the nodes. You can see the list of commands by running:

```bash
cargo run -p chord-rs-cli -- --help
```
