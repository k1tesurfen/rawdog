# Architecture: rawdog

## Overview
`rawdog` is a high-performance RAW photo culling tool for macOS. It uses a Rust backend (Tauri) for heavy lifting and a minimal TypeScript/Vite frontend for the UI.

## Backend (Rust)
- **CLI Parsing:** Captures the target directory from the command line.
- **Scanning:** Uses `walkdir` to find RAW files (ARW, CR2, NEF, etc.).
- **Extraction:** Uses macOS `sips` to extract JPEG previews into a cache folder (`~/.cache/rawdog/<uuid>`).
- **State:** Maintains a list of images and their culling status (Pending, Kept, Rejected, Favorite).
- **Commands:** IPC bridge for the frontend to get images and update status.

## Frontend (TypeScript)
- **UI:** Fullscreen image display with minimal overlays.
- **Controls:** Keyboard-driven (`→`, `←`, `J`, `K`, `F`, `X`, `U`).
- **Optimization:** Preloads the next image in the sequence to ensure zero-lag transitions.
- **Asset Loading:** Uses Tauri's `asset` protocol to load JPEGs directly from the local cache.

## Data Flow
1. User runs `rawdog ~/shoot`.
2. Backend identifies images and starts background extraction.
3. Frontend requests image list via `get_images`.
4. User swipes through images; status updates are sent via `update_status`.
5. On exit, backend can perform final cleanup (e.g., moving rejected files).
