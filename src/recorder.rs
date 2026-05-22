// src/recorder.rs
use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use hound::{WavWriter, WavSpec};
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use ctrlc;
/// List all available input devices and print their names.
pub fn list_input_devices() -> Result<()> {
    let host = cpal::default_host();
    println!("Available input devices:");
    for (idx, device) in host.input_devices()?.enumerate() {
        let name = device.name().unwrap_or_else(|_| "<unknown>".to_string());
        println!("  {}: {}", idx + 1, name);
    }
    Ok(())
}

/// Get an input device by its 1‑based index as shown by `list_input_devices`.
pub fn get_input_device_by_index(index: usize) -> Result<cpal::Device> {
    let host = cpal::default_host();
    let mut devices = host.input_devices()?;
    let mut i = 0usize;
    for device in devices {
        i += 1;
        if i == index {
            return Ok(device);
        }
    }
    Err(anyhow!("No input device with index {}", index))
}



/// Select the default input device. For now we use the system default.
/// In the future this could scan devices for activity level.
pub fn select_active_input_device() -> Result<cpal::Device> {
    let host = cpal::default_host();
    // Try default first
    if let Some(device) = host.default_input_device() {
        return Ok(device);
    }
    // Fallback: iterate over devices and pick the first available
    match host.input_devices() {
        Ok(mut devices) => {
            if let Some(device) = devices.next() {
                return Ok(device);
            }
            Err(anyhow!("No audio input device found"))
        }
        Err(e) => Err(anyhow!("Failed to enumerate input devices: {}", e)),
    }
}


/// Record audio from the given input device into a WAV file.
/// The function runs until the user presses Ctrl‑C.
pub fn record_to_wav(device: &cpal::Device, output_path: &Path) -> Result<()> {
    // Choose a supported input config with a usable sample format (F32, I16, U16).
    let desired_formats = [SampleFormat::F32, SampleFormat::I16, SampleFormat::U16];
    let supported_config = device
        .supported_input_configs()?
        .filter(|c| desired_formats.contains(&c.sample_format()))
        .max_by_key(|c| c.max_sample_rate())
        .ok_or_else(|| anyhow!("No supported input config with a usable sample format"))?
        .with_max_sample_rate();
    let config: StreamConfig = supported_config.clone().into();
    let sample_format = supported_config.sample_format();
    // Use 16‑bit PCM for WAV regardless of device format
    let bits_per_sample = 16;
    let wav_sample_format = hound::SampleFormat::Int;
    let spec = WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample,
        sample_format: wav_sample_format,
    };

    let writer = WavWriter::create(output_path, spec)?;
    let writer = Arc::new(std::sync::Mutex::new(writer));
    let is_recording = Arc::new(AtomicBool::new(true));

    // Set up Ctrl-C handler to stop recording
    {
        let is_recording = Arc::clone(&is_recording);
        ctrlc::set_handler(move || {
            is_recording.store(false, Ordering::SeqCst);
        })?;
    }

    // Build the input stream based on sample format
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let is_rec = Arc::clone(&is_recording);
    let writer_arc = Arc::clone(&writer);
    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !is_rec.load(Ordering::SeqCst) {
                    return;
                }
                let mut writer = writer_arc.lock().unwrap();
                for &sample in data {
                    // Convert f32 [-1.0, 1.0] to i16
                    let s = (sample * i16::MAX as f32) as i16;
                    writer.write_sample(s).ok();
                }
            },
            err_fn,
            None,
        )?,
        SampleFormat::I16 => {
            let is_rec = Arc::clone(&is_recording);
            let writer_arc = Arc::clone(&writer);
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if !is_rec.load(Ordering::SeqCst) {
                        return;
                    }
                    let mut writer = writer_arc.lock().unwrap();
                    for &sample in data {
                        writer.write_sample(sample).ok();
                    }
                },
                err_fn,
                None,
            )?
        },
        SampleFormat::U16 => {
            let is_rec = Arc::clone(&is_recording);
            let writer_arc = Arc::clone(&writer);
            device.build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    if !is_rec.load(Ordering::SeqCst) {
                        return;
                    }
                    let mut writer = writer_arc.lock().unwrap();
                    for &sample in data {
                        // Convert unsigned 16-bit to signed 16-bit for WAV
                        writer.write_sample(sample as i16).ok();
                    }
                },
                err_fn,
                None,
            )?
        },
        _ => return Err(anyhow!("Unsupported sample format for stream")),
    };

    // Start playback
    stream.play()?;
    println!("Recording... Press Ctrl‑C to stop.");
    // Busy‑wait until Ctrl‑C sets the flag
    while is_recording.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Dropping the stream stops audio capture
    drop(stream);
    // Flush WAV file (finalize header)
    writer.lock().unwrap().flush()?;
    Ok(())
}
