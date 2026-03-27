# REQUIREMENT.md — What Voix Must Do

## Product Vision

Voix is an open-source, local-first voice input tool with AI-powered text polishing. Speak naturally into any application, and Voix transforms your speech into clean, polished text — removing filler words, fixing repetition, and adapting to your personal style.

Target: macOS and Windows, single developer release.

---

## User Stories

### Core Flow

**As a user, I want to:** Speak naturally while Voix captures and polishes my speech, then have the result typed into whatever application I'm using — without switching apps or changing how I type.

**Trigger:** Global hotkey (default: Option+E on macOS, Alt+E on Windows) activates voice capture.

**End state:** Polished text is injected into the active application at cursor position.

---

### Voice Capture & Transcription

| ID | Requirement | Priority |
|----|-------------|----------|
| V-1 | App runs as a menu bar / system tray application | Must |
| V-2 | Global hotkey activates voice capture from system microphone | Must |
| V-3 | Press hotkey again or Esc to stop capture | Must |
| V-4 | Voice transcribed locally using Whisper (no cloud upload of audio) | Must |
| V-5 | Transcription latency < 500ms for typical sentence | Should |
| V-6 | Support 100+ languages; auto-detect language | Should |
| V-7 | Works with any microphone (system default) | Must |
| V-8 | Visual feedback during recording (indicator in menu bar) | Must |
| V-9 | Audio level meter while recording | Should |

---

### AI Polish

| ID | Requirement | Priority |
|----|-------------|----------|
| P-1 | Transcribed text passed through LLM for polishing | Must |
| P-2 | Remove filler words: "um", "uh", "like", "you know", "I mean" | Must |
| P-3 | Remove redundant repetitions ("the the", "I I I") | Must |
| P-4 | Keep user's intended meaning and voice intact | Must |
| P-5 | Auto-detect correction mid-sentence ("no wait — actually") | Should |
| P-6 | Format lists and structured content | Should |
| P-7 | Support local LLM (Ollama) with fallback to API | Should |
| P-8 | LLM polish latency < 1s for typical sentence | Should |
| P-9 | User can configure polish aggressiveness: light / medium / strong | Could |

---

### Text Injection

| ID | Requirement | Priority |
|----|-------------|----------|
| T-1 | Inject polished text into currently focused application | Must |
| T-2 | Simulate keyboard input (not clipboard paste) | Must |
| T-3 | Maintain user's typing cursor position after injection | Must |
| T-4 | Support all standard macOS / Windows applications | Must |
| T-5 | If injection fails, show error and offer to copy to clipboard | Must |
| T-6 | Undo last injection (Ctrl+Z / Cmd+Z) by restoring previous clipboard | Could |

---

### Personalization

| ID | Requirement | Priority |
|----|-------------|----------|
| S-1 | Learn user's writing style over time (stored locally) | Could |
| S-2 | Personal dictionary for names/terms Voix should recognize | Could |
| S-3 | Per-app voice profiles (work email vs casual chat = different polish) | Could |
| S-4 | All personalization data stays local (no cloud sync) | Must |

---

### UI / UX

| ID | Requirement | Priority |
|----|-------------|----------|
| U-1 | Menu bar / system tray icon with status indicator | Must |
| U-2 | Settings panel: hotkey, language, LLM provider, polish level | Must |
| U-3 | History view: last N transcriptions with edit/retry | Should |
| U-4 | Dark mode support | Should |
| U-5 | Onboarding: first-run permission request (mic, accessibility) | Must |
| U-6 | Show processing status (recording → transcribing → polishing → injecting) | Must |

---

## Out of Scope (v1)

- Mobile apps (iOS, Android)
- Team collaboration / cloud sync
- Video / screen recording
- Custom wake word (always hotkey-triggered)
- Browser extension version
- Real-time conversation transcription

---

## Success Metrics (v1)

- Transcription accuracy: > 95% on clean English audio
- End-to-end latency (speak → inject): < 2 seconds on Apple Silicon / modern PC
- Crash rate: < 0.1% over 1000 sessions
- User satisfaction: "significantly faster than typing" self-reported > 80%
