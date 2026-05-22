// src/main.rs
use anyhow::{Result, anyhow};
use env_logger::Env;
use log::{info, error};
use std::env;
use std::path::PathBuf;

use cpal::traits::DeviceTrait;

mod recorder;

fn print_usage() {
    eprintln!("Usage: audio_recorder [output_path] [--list-devices] [--auto-signal] [--device-index <index>]");
    eprintln!("If no output_path is provided, 'recording.wav' is created in the current directory.");
}

fn main() -> Result<()> {
    // Initialize logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut device_index: Option<usize> = None;
    let mut auto_signal = false;
    let mut output_path = PathBuf::from("recording.wav");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list-devices" => {
                recorder::list_input_devices()?;
                return Ok(());
            }
            "--auto-signal" => {
                auto_signal = true;
            }
            "--device-index" => {
                if i + 1 < args.len() {
                    device_index = Some(args[i + 1].parse::<usize>().map_err(|_| anyhow!("Invalid device index"))?);
                    i += 1;
                } else {
                    return Err(anyhow!("--device-index requires a numeric argument"));
                }
            },
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            other => {
                // First non-flag argument is treated as output path
                if output_path == PathBuf::from("recording.wav") && !other.starts_with("--") {
                    output_path = PathBuf::from(other);
                }
            }
        }
        i += 1;
    }

    // Choose device
    let device = if let Some(idx) = device_index {
        recorder::get_input_device_by_index(idx)?
    } else if auto_signal {
        recorder::select_device_with_signal(0.01)? // RMS threshold
    } else {
        recorder::select_active_input_device()? // fallback to default or first available
    };
    info!("Selected device: {}", device.name()?);

    // Record until Ctrl‑C is pressed
    match recorder::record_to_wav(&device, &output_path) {
        Ok(_) => {
            info!("Recording finished, file saved to {}", output_path.display());
            Ok(())
        }
        Err(e) => {
            error!("Recording failed: {:?}", e);
            Err(e)
        }
    }
}
