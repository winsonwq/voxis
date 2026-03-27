# ARCHITECT.md — Voix System Design

## Overview

Voix is a **local-first, AI-powered voice input tool** built as a menu bar / system tray application. The core flow is:

```
User presses hotkey
  → Audio captured from microphone
  → Whisper (local) transcribes speech → text
  → LLM (local or API) polishes text
  → Text injected into active application via keyboard simulation
```

---

## Technology Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Framework** | Tauri 2.x | Native app feel, Rust backend, lighter than Electron |
| **UI** | React + TypeScript | Fast iteration, good component ecosystem |
| **Audio Capture** | cpal / alsa-sys | Cross-platform, low latency |
| **STT** | Whisper.cpp | Fully local, no cloud, fast on CPU/GPU |
| **LLM** | Ollama (local) + OpenAI API (fallback) | User choice, respects privacy |
| **Text Injection** | CGEvent (macOS) / SendInput (Windows) | Native keyboard simulation |
| **Hotkey** | rdev / enigo | Cross-platform global hotkey registration |
| **Persistence** | SQLite (via rusqlite) | Local user data, no cloud |

---

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Menu Bar UI (React)                  │
│  Status │ Hotkey indicator │ Settings │ History         │
└──────────────────┬──────────────────────────────────────┘
                   │ Tauri IPC (invoke)
┌──────────────────▼──────────────────────────────────────┐
│                 Rust Backend (Tauri)                     │
│                                                          │
│  ┌──────────────┐   ┌──────────────┐   ┌─────────────┐  │
│  │ AudioManager │ → │ WhisperEngine│ → │ LLMPolisher │  │
│  │ (capture)    │   │ (transcribe) │   │ (polish)    │  │
│  └──────────────┘   └──────────────┘   └─────────────┘  │
│         │                                    │           │
│         └──────────┬────────────────────────┘           │
│                    ▼                                     │
│         ┌──────────────────┐                             │
│         │ TextInjector     │                             │
│         │ (CGEvent/SendIn)│                             │
│         └──────────────────┘                             │
│                    │                                     │
│         ┌──────────▼──────────┐                          │
│         │ SettingsStore      │  ← SQLite                 │
│         │ (user preferences) │                          │
│         └────────────────────┘                          │
└─────────────────────────────────────────────────────────┘
```

---

## Module Responsibilities

### AudioManager

- Initializes and manages microphone stream
- Handles sample rate conversion (16kHz for Whisper)
- Buffers audio frames during recording
- Emits audio data to WhisperEngine

**Key API:**
```rust
fn start_capture() -> Result<AudioStream>
fn stop_capture() -> Result<Vec<f32>>  // returns PCM samples
```

### WhisperEngine

- Loads Whisper model on startup (downloaded once, cached)
- Accepts PCM audio and returns transcribed text
- Supports model selection: tiny / base / small / medium
- CPU + GPU (Metal / CUDA) support

**Key API:**
```rust
fn transcribe(samples: &[f32]) -> Result<String>
fn download_model(model: ModelSize) -> Result<PathBuf>
```

### LLMPolishEngine

- Sends transcribed text to LLM with polish prompt
- Supports Ollama (local) and OpenAI API
- Configurable aggressiveness: light / medium / strong
- Returns polished text

**Key API:**
```rust
fn polish(text: &str, level: PolishLevel) -> Result<String>
fn set_provider(provider: LLMProvider)
```

### TextInjector

- Simulates keyboard events to inject text
- Uses CGEvent on macOS, SendInput on Windows
- Handles special characters and Unicode
- Falls back to clipboard if direct injection fails

**Key API:**
```rust
fn inject(text: &str) -> Result<()>
fn supports_injection() -> bool
```

### SettingsStore

- SQLite-backed key-value store
- Persists: hotkey, language, LLM provider, polish level, personal dictionary
- Migrations for schema updates

**Key API:**
```rust
fn get<T: Deserialize>(key: &str) -> Result<Option<T>>
fn set<T: Serialize>(key: &str, value: &T) -> Result<()>
```

---

## Data Flow

```
Hotkey pressed
    │
    ▼
AudioManager.start_capture()
    │ (user speaks...)
    ▼
AudioManager.stop_capture() → Vec<f32>
    │
    ▼
WhisperEngine.transcribe(audio) → "um so I was thinking maybe..."
    │
    ▼
LLMPolishEngine.polish(text, medium)
    → "So I was thinking maybe..."
    │
    ▼
TextInjector.inject(polished_text)
    │ (Ctrl+V simulation)
    ▼
Result: text appears in target app
```

---

## Platform-Specific Notes

### macOS

- Menu bar app (NSStatusItem)
- Accessibility permission required for global hotkey + CGEvent
- Microphone permission required
- Metal GPU acceleration for Whisper via MLX or whisper.cpp Metal backend
- App Store distribution NOT planned for v1 (direct download)

### Windows

- System tray app ( NotifyIcon)
- Input injection via SendInput (no admin required for user-level apps)
- CUDA support for Whisper (via whisper.cpp CUDA backend)
- Portable executable (no installer required)

---

## File Structure

```
voxis/
├── src/                    # React frontend
│   ├── components/         # UI components
│   ├── hooks/             # React hooks (useAudio, useSettings)
│   ├── stores/            # State management
│   └── App.tsx
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── audio.rs        # AudioManager
│   │   ├── whisper.rs      # WhisperEngine
│   │   ├── polish.rs       # LLMPolishEngine
│   │   ├── inject.rs       # TextInjector
│   │   ├── settings.rs     # SettingsStore
│   │   └── hotkey.rs       # Global hotkey registration
│   ├── Cargo.toml
│   └── tauri.conf.json
├── models/                 # Whisper model files (downloaded at runtime)
├── SKILLS/                 # AI harness skill definitions
├── docs/                   # Requirements, architecture, design docs
├── AGENTS.md               # AI harness definitions
├── SKILL.md               # Skill loader
├── .gitignore
├── README.md
└── LICENSE (MIT)
```

---

## Non-Functional Requirements

| Requirement | Target |
|-------------|--------|
| Startup time | < 2 seconds |
| Memory usage (idle) | < 100MB |
| Memory usage (recording) | < 300MB |
| Battery impact | Minimal (no background polling) |
| Whisper model size (small) | ~140MB |
| Offline capability | Full (no network required for core flow) |
