/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Implant module - the payload that runs on compromised systems.
//!
//! The implant is a small program that:
//! 1. Checks in with the C2 server periodically
//! 2. Receives commands from the server
//! 3. Executes commands locally
//! 4. Reports results back to the server
//!
//! ## Design Goals
//!
//! - **Small**: Minimal dependencies, small binary size
//! - **Stealthy**: Runs in background, minimal footprint
//! - **Resilient**: Handles network errors, retries connections
//! - **Configurable**: C2 URL and settings can be baked in at compile time

pub mod client;

pub use client::*;
