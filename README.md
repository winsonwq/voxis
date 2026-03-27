# Voix

> Open source Typeless alternative — AI voice input with real-time polish.

**Voix** is a local-first menu bar application that transforms your speech into polished, professional text in any application. Speak naturally, and Voix handles the rest.

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## What It Does

1. **Press a hotkey** (Option+E on macOS, Alt+E on Windows)
2. **Speak** — your words are transcribed locally (Whisper)
3. **AI polishes** — removes filler words, fixes repetitions, formats lists
4. **Text appears** in your target application

All processing happens on your machine. No cloud. No subscription. No data leaving your computer.

---

## Features

- 🎙️ **Local transcription** — Whisper.cpp, runs entirely offline
- ✨ **AI text polishing** — removes "um", "uh", "like", repetitions
- 🔒 **100% private** — audio never leaves your machine
- 🌍 **100+ languages** — speak in any language, auto-detected
- 🖥️ **Works everywhere** — any app, any text field
- 🤖 **Local LLM** — Ollama integration, or use OpenAI API
- 🌙 **Dark mode** — native look and feel

---

## Status

**Early development.** See [REQUIREMENT.md](REQUIREMENT.md) for what's being built.

---

## Architecture

```
Menu Bar App (Tauri + React)
    │
    ├── Audio Capture (system microphone)
    │       │
    │       ▼
    ├── Whisper.cpp (local STT)
    │       │
    │       ▼
    ├── LLM Polish (Ollama or OpenAI)
    │       │
    │       ▼
    └── Text Injection (keyboard simulation)
```

Full system design in [ARCHITECT.md](ARCHITECT.md).

---

## AI Harness

Voix is built with an **AI harness** — the AI can write its own code, run its own tests, and improve itself within defined boundaries.

See [AGENTS.md](AGENTS.md) for the harness specification.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri 2.x |
| UI | React + TypeScript |
| STT | Whisper.cpp |
| LLM | Ollama / OpenAI API |
| Audio | cpal |
| Text injection | CGEvent (macOS) / SendInput (Windows) |

---

## Getting Started

### Prerequisites

- macOS 12+ or Windows 10+
- Rust 1.70+
- Node.js 18+

### Setup

```bash
# Clone the repo
git clone https://github.com/winsonwq/voxis.git
cd voxis

# Install dependencies
npm install

# Download Whisper model (first run will also prompt this)
# Recommended: "small" model for balance of speed/accuracy
./scripts/download-model.sh small

# Run in development
npm run tauri dev
```

### Ollama Setup (for local LLM)

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull a model
ollama pull qwen2.5:3b

# Start Ollama (runs on port 11434 by default)
ollama serve
```

---

## Contributing

Voix is an open project. Read [AGENTS.md](AGENTS.md) to understand the AI harness workflow, then:

1. Fork and clone
2. Create a branch: `git checkout -b feature/your-feature`
3. Make changes, run tests
4. Open a PR

For large changes, open an issue first to discuss direction.

---

## License

MIT — do whatever you want with it.
