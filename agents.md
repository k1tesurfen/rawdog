# agents.md

## Project: Fast Photo Culling System (macOS)

## Overview

This project is a macOS-native, terminal-first photo culling tool designed for rapid image rating after a shoot.

It enables photographers to quickly triage large sets of RAW images using a keyboard-driven, swipe-style decision workflow in a near-fullscreen browser UI.

The system prioritizes:

- extreme interaction speed during culling
- minimal cognitive overhead
- zero reliance on heavy photo editing suites
- filesystem-native workflow
- deterministic, cache-driven performance

---

## Core Philosophy

### Interaction speed > everything

No decoding, no IO, no blocking once UI starts.

### Startup can be slow (20–30s acceptable)

Used for:

- preview extraction
- caching
- indexing

### RAW files are not edited

This is a decision engine, not an editor.

---

## Workflow

rawdog ~/photos/shoot

1. Rust backend scans directory
2. Extracts embedded JPEG previews (LibRaw)
3. Builds cache + index
4. Starts local server (careful with port. we usually have 80 and 8080 already in use)
5. Browser opens automatically (or link is presented)

---

## UI

- fullscreen image
- black background
- minimal overlay

### Controls

→ / D = keep  
← / A = reject  
J/K = next/prev  
F = favorite  
X = delete  
U = undo

---

## Architecture

CLI → Rust backend → Local server → Browser UI

---

## Tech Stack

- Tauri (macOS only)
- Rust backend
- React or minimal TS frontend
- LibRaw for preview extraction

---

## Performance Model

Startup (slow allowed):

- scan
- extract previews
- cache warmup

Runtime (critical):

- no decoding
- no disk IO
- instant image swap

---

## Success Criteria

- 1000 images in <10 min culling
- zero perceptible lag
- uninterrupted flow state
