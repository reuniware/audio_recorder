# Audio Recorder

A small **Windows‑only** Rust command‑line tool that records audio from the default (or a user‑selected) microphone and saves it as a 16‑bit PCM WAV file.

---

## 📦 Prerequisites

- **Rust toolchain** (stable) – install with [rustup](https://rustup.rs/).
- **Microsoft Visual C++ Build Tools** (MSVC) – required for `cpal` on Windows. The standard Rust installer pulls the appropriate toolchain, but you may need the Windows SDK if you haven’t installed it before.
- **Git** (optional, for cloning the repository).

---

## 🔨 Building the binary

```bash
# Clone the repository (if you haven’t already)
git clone https://github.com/reuniware/audio_recorder.git
cd audio_recorder

# Build in release mode (optimized binary)
cargo build --release
```

The compiled executable will be placed at:
```
<project‑root>/target/release/audio_recorder.exe
```

> **Tip:** You can copy the binary to any folder you like (e.g. `C:\Program Files\audio_recorder`) or run it directly from the `target/release` directory.

---

## 📋 Command‑line options

| Option | Description |
|--------|-------------|
| `--list-devices` | Lists all input devices (microphones) available on the system. |
| `--device-index <N>` | Selects the *N*‑th device from the list (1‑based). If omitted, the program uses the system default device. |
| `--auto-signal` | Enables automatic device selection based on the highest audio signal level (RMS detection). |
| `<output‑file>` | Optional path for the resulting WAV file. Defaults to `recording.wav` in the current directory. |

### Example usage

```powershell
# List microphones
& "...\audio_recorder.exe" --list-devices

# Record using the default microphone (output → recording.wav)
& "...\audio_recorder.exe"

# Record using a specific device (index 2) and custom filename
& "...\audio_recorder.exe" --device-index 2 my_record.wav
```

Press **Ctrl‑C** to stop the recording.

---

## 📂 Output format

- **WAV** (RIFF) file
- 16‑bit PCM, 44.1 kHz (standard CD quality)
- Stereo/mono depending on the selected device’s channel count

---

## 🛠️ Common issues & troubleshooting

- **"No default input device found"** – Ensure a microphone is connected and not disabled in Windows Sound settings.
- **"Unsupported sample format for stream"** – This occurs when the device reports an unusual format; the tool now normalises all formats to 16‑bit PCM, so the error should not appear after the latest update.
- **Line‑ending warnings (`LF will be replaced by CRLF`)** – These are purely Git warnings on Windows; they do not affect the binary.

---

## 🤝 Contributing

Feel free to fork the repository, open issues, or submit pull requests. The project uses the following crates:
- `cpal` – cross‑platform audio I/O
- `hound` – WAV file writing
- `anyhow` – error handling
- `ctrlc` – graceful shutdown on Ctrl‑C

---

## 📜 License

MIT – see the `LICENSE` file in the repository.
