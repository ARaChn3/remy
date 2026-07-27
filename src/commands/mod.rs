/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Command handlers for the implant.
//!
//! Each module implements handlers for different command categories.
//! Commands are dispatched from the implant's main loop based on
//! the CommandType received from the C2 server.

pub mod exec;
pub mod files;
pub mod sysinfo;
pub mod screenshot;
pub mod troll;
pub mod persist;

pub use exec::*;
pub use files::*;
pub use sysinfo::*;
pub use screenshot::*;
pub use persist::*;
