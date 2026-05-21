# Installation & Setup Guide

This guide covers how to set up your environment to build and run `rawdog` on macOS.

## Prerequisites

1.  **Node.js & npm:** Required for the frontend and Tauri CLI.
    -   Download from [nodejs.org](https://nodejs.org/) or install via Homebrew: `brew install node`.
2.  **Rust & Cargo:** The engine behind `rawdog`.
    -   Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
    -   **Important:** After installation, ensure Cargo is in your PATH. You might need to restart your terminal or run `source $HOME/.cargo/env`.
3.  **macOS Build Tools:** Required for compiling Rust apps on macOS.
    -   Install XCode Command Line Tools: `xcode-select --install`.

## Project Setup

1.  **Clone the repository:**
    ```bash
    git clone <repo-url>
    cd rawdog
    ```

2.  **Install Dependencies:**
    ```bash
    npm install
    ```

## Running the App

### Development Mode
To run `rawdog` with Hot Module Replacement (HMR) and debug logs:
```bash
npm run tauri dev -- /path/to/your/photos
```

### Building for Production
To create a standalone macOS `.app` bundle:
```bash
npm run tauri build
```
The resulting app will be in `src-tauri/target/release/bundle/macos/`.

## Troubleshooting "Cargo Not Found"

If you encounter an error like `cargo: command not found` even after installing Rust, follow these steps:

1.  **Verify Installation:** Run `ls ~/.cargo/bin`. You should see `cargo`, `rustc`, etc.
2.  **Update your Shell Profile:** Depending on your shell (zsh is default on modern macOS), add this line to your `~/.zshrc` or `~/.bash_profile`:
    ```bash
    export PATH="$HOME/.cargo/bin:$PATH"
    ```
3.  **Refresh Shell:** Run `source ~/.zshrc` (or restart the terminal).
4.  **Test:** Run `cargo --version`. If it prints a version number, you are ready to go.

## Why macOS `sips`?

`rawdog` uses the built-in macOS `sips` (Scriptable Image Processing System) tool to extract previews. 
- **Zero Dependencies:** No need to install complex C-libraries like LibRaw.
- **Hardware Acceleration:** `sips` uses the same engine as Finder and Preview, making it extremely fast on Apple Silicon and Intel Macs.
- **Stability:** It handles almost all RAW formats supported by macOS natively.
