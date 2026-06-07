# MeshCoreGRPCGateway

A snap that serves a gRPC server for controlling the MeshCore system on a MeshHat.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Setting Dependencies For Snapping](#setting-dependencies-for-snapping)
- [Installing Dependencies](#installing-dependencies)
- [Building the Project](#building-the-project)
- [Installing the Snap](#installing-the-snap)

## Prerequisites

Before you begin, ensure you have the following installed on your system:

### Required Tools

- **Cargo/Rustup** - The package and toolchain managers for rust
- **Just** - A handy way to save and run project-specific commands
- **Snapcraft** - The build system for creating snaps
- **LXD** Used for building in isolation of a remote system

## Setting Dependencies For Snapping

### Installing and Initializing LXD and Snapcraft

If you don't have LXD and snapcraft installed, install and initialize them first:

``` bash
sudo snap install lxd --classic
sudo snap install snapcraft --classic
sudo lxd init --auto
sudo usermod -a -G lxd $USER newgrp lxd
```

## Installing Dependencies

Install rust cargo (the package manager for rust) and just:

``` bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh source $HOME/.cargo/env
cargo install just
```

## Building the Project

To build the project directly with cargo:

``` bash
cd meshcore-grpc-gateway && cargo build --release
```

## Building the Snap

To build the snap on a Linux system (arm64 / amd64), run `snapcraft`. This will create a `.snap`
 file in the current directory. The build process will:

## Installing the Snap

### From the Snap Store

Install the snap directly from the Snap Store:

``` bash
sudo snap install meshcore-grpc-gateway
```

### From a Locally Built Snap

After building the snap locally, install it with:

``` bash
sudo snap install --dangerous ./meshcore-grpc-gateway_*.snap
```

The `--dangerous` flag is required for locally built snaps that aren't signed.
 Replace `meshcore-grpc-gateway_*.snap` with the actual filename of your built snap.

## Usage

Once installed, the gRPC server will be available for controlling the MeshCore system on your MeshHat.

For more information on using the snap, check the snap's documentation:

``` bash
snap run meshcore-grpc-gateway --help
```

## Contributing

Contributions are welcome! Please ensure all code is properly tested and follows Rust best practices before submitting a pull request.
