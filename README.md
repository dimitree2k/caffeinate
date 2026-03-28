# Caffeinate for Windows

A lightweight system tray app that prevents your PC from sleeping — like macOS `caffeinate`, but for Windows.

- single native exe
- no .NET, no runtime
- minimal footprint (~1.2 MB, <1 MB RAM)

## Install

There is no installer. Download `caffeinate.exe` from the [Releases](../../releases) page, put it wherever you want, and run it. That's it.

No registry entries, no services, no dependencies, no setup wizard. One file — delete it when you're done.

Windows SmartScreen may warn on first launch — click **"More info" → "Run anyway"**.

## Features

- **Keep Awake** — toggle to prevent sleep and display off indefinitely
- **Timer** — keep awake for 15 min / 30 min / 1 hour / 2 hours, or enter a custom duration (1–1440 min). Shows a notification when the timer expires.
- **Black Out Screen** — fills all monitors with a black screen. Any key or click dismisses it and locks the workstation.
- **System tray** — lives in your tray, left- or right-click to open the menu. Tooltip shows current state.
- **Single instance** — launching a second copy silently exits.

No admin rights required.

## Build from source

If you don't want to download a pre-built binary, build it yourself:

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- For cross-compilation from Linux/WSL:
  - MinGW: `sudo apt install gcc-mingw-w64-x86-64`
  - Windows target: `rustup target add x86_64-pc-windows-gnu`

### Build

**On Windows (native):**

```
cargo build --release
```

Output: `target/release/caffeinate.exe`

**On Linux / WSL (cross-compile):**

```
cargo build --target x86_64-pc-windows-gnu --release
```

Output: `target/x86_64-pc-windows-gnu/release/caffeinate.exe`

### Custom icon

Replace `caffeinate.ico` with your own `.ico` file (16×16 + 32×32, 32-bit RGBA) and rebuild. The icon is embedded at compile time.

## License

[MIT](LICENSE)
