# MeshCoreGRPCGateway Roadmap

This document outlines the current state and planned features for the MeshCoreGRPCGateway project.

## Current State

### Completed Features

- **CLI Tool** - Full-featured command-line interface for interacting with the MeshCore system
- **gRPC Service** - Robust gRPC server implementation for remote control and management
- **Message Transport** - Ability to send and receive messages through private devices and channels
- **Snapcraft Integration** - Working `snapcraft.yaml` configuration for building and distributing the snap
- **Snap Packaging** - Complete support for creating installable snap packages

## Upcoming Features

### Remote DFU (Device Firmware Update)

Implement Device Firmware Update capabilities firmware updates for MeshCore devices without requiring physical access to the hardware.

**Goals:**

- Enable remote DFU through the gRPC gateway
- Progress tracking and status reporting
- Rollback capabilities for failed updates
- DFU will be backed by OpenOCD

### GPIO Reset Support

Add support for hardware reset functionality through GPIO (General Purpose Input/Output) control on MeshHat devices.

**Goals:**

- Implement GPIO pin control through the gRPC API
- Support for hardware reset sequences
- Status monitoring for reset operations

## Planned Enhancements

### Phase 1: Core Functionality Expansion

- [ ] Remote DFU implementation
- [ ] GPIO Reset support
- [ ] Enhanced error handling and diagnostics
- [ ] Improved logging and debugging capabilities

## Ongoing Maintenance

- Dependency updates and security patches
- Bug fixes and performance improvements
- Community feedback and issue resolution
- Test coverage improvements
- Documentation updates

## Contributing

We welcome community contributions! If you're interested in working on any of these features, please open an issue or submit a pull request.

Last Updated: June 2026
