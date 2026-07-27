/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Session management for connected implants.
//!
//! Tracks which implants are connected, their metadata, queued
//! commands, and command results. Uses in-memory storage for
//! simplicity (a real C2 might use a database).

use crate::shared::{Command, CommandResult, CommandType, ImplantInfo};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Manages all implant sessions and their associated data.
///
/// This is the central state store for the C2 server. It's wrapped
/// in Arc<RwLock<>> to allow safe concurrent access from multiple
/// HTTP handler threads.
#[derive(Debug, Default)]
pub struct SessionManager {
    /// Connected implants, keyed by implant ID
    implants: HashMap<String, ImplantInfo>,

    /// Commands waiting to be sent to implants
    /// Key: implant_id, Value: list of pending commands
    pending_commands: HashMap<String, Vec<Command>>,

    /// Results received from implants
    /// Key: command_id, Value: result
    results: HashMap<String, CommandResult>,
}

/// Thread-safe wrapper around SessionManager.
///
/// Use this type for sharing the session manager across async tasks.
pub type SharedSessions = Arc<RwLock<SessionManager>>;

impl SessionManager {
    /// Creates a new empty session manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new session manager wrapped in Arc<RwLock<>>.
    ///
    /// This is the preferred way to create a session manager for
    /// use with the HTTP server.
    pub fn shared() -> SharedSessions {
        Arc::new(RwLock::new(Self::new()))
    }

    /// Registers or updates an implant.
    ///
    /// Called when an implant checks in. If the implant is new,
    /// it's added to the session list. If it already exists, its
    /// last_seen timestamp is updated.
    ///
    /// # Arguments
    /// * `info` - Implant information from check-in
    pub fn register_implant(&mut self, mut info: ImplantInfo) {
        // Update last_seen time
        info.last_seen = Utc::now();

        // If new implant, set first_seen as well
        if !self.implants.contains_key(&info.id) {
            info.first_seen = Utc::now();
            // Initialize empty command queue
            self.pending_commands.insert(info.id.clone(), Vec::new());
        }

        self.implants.insert(info.id.clone(), info);
    }

    /// Gets information about a specific implant.
    pub fn get_implant(&self, id: &str) -> Option<&ImplantInfo> {
        self.implants.get(id)
    }

    /// Gets all connected implants.
    pub fn get_all_implants(&self) -> Vec<&ImplantInfo> {
        self.implants.values().collect()
    }

    /// Queues a command for an implant.
    ///
    /// The command will be delivered the next time the implant
    /// checks in with the server.
    ///
    /// # Arguments
    /// * `implant_id` - Target implant
    /// * `command_type` - The command to execute
    ///
    /// # Returns
    /// The command ID for tracking
    pub fn queue_command(&mut self, implant_id: &str, command_type: CommandType) -> Option<String> {
        // Check implant exists
        if !self.implants.contains_key(implant_id) {
            return None;
        }

        let command = Command {
            id: Uuid::new_v4().to_string(),
            implant_id: implant_id.to_string(),
            command: command_type,
            created_at: Utc::now(),
        };

        let cmd_id = command.id.clone();

        // Add to pending queue
        self.pending_commands
            .entry(implant_id.to_string())
            .or_default()
            .push(command);

        Some(cmd_id)
    }

    /// Gets and clears pending commands for an implant.
    ///
    /// Called during check-in to deliver commands. Commands are
    /// removed from the pending queue once retrieved.
    ///
    /// # Arguments
    /// * `implant_id` - The implant requesting commands
    ///
    /// # Returns
    /// Vector of commands to execute
    pub fn get_pending_commands(&mut self, implant_id: &str) -> Vec<Command> {
        self.pending_commands
            .get_mut(implant_id)
            .map(|cmds| std::mem::take(cmds))
            .unwrap_or_default()
    }

    /// Stores a command result.
    ///
    /// Called when an implant reports back after executing a command.
    ///
    /// # Arguments
    /// * `result` - The command execution result
    pub fn store_result(&mut self, result: CommandResult) {
        self.results.insert(result.command_id.clone(), result);
    }

    /// Gets a command result by command ID.
    pub fn get_result(&self, command_id: &str) -> Option<&CommandResult> {
        self.results.get(command_id)
    }

    /// Gets all results for a specific implant.
    pub fn get_implant_results(&self, implant_id: &str) -> Vec<&CommandResult> {
        self.results
            .values()
            .filter(|r| r.implant_id == implant_id)
            .collect()
    }

    /// Removes an implant and all associated data.
    ///
    /// Use when an implant has been cleaned up or is no longer
    /// responding.
    pub fn remove_implant(&mut self, id: &str) {
        self.implants.remove(id);
        self.pending_commands.remove(id);
        // Keep results for historical purposes
    }
}
