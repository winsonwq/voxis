# DESIGN.md — Voix Design Language & UX

## Design Philosophy

**"Invisible tool, visible result."** Voix should feel like an extension of your thoughts — present when you need it, gone when you don't. The UI is a thin, unobtrusive layer on top of a powerful engine.

The voice is yours. The polish is yours. The data is yours.

---

## Visual Identity

### Aesthetic Direction

Clean, functional, developer-tool aesthetic. Think: Raycast meets a high-end audio app. Dark mode first, light mode supported.

- **Backgrounds:** Deep dark grays (#0D0D0F, #1A1A1D) for dark mode; clean whites (#FAFAFA, #F5F5F5) for light
- **Accent:** Electric violet (#7C3AED) — used sparingly for active states and CTAs
- **Success / Recording:** Coral red (#FF6B6B) for recording indicator
- **Text:** High contrast whites/blacks with careful use of muted grays for secondary info

### Color Palette

```
Dark Mode:
  background:     #0D0D0F
  surface:        #1A1A1D
  surface-hover:  #252529
  border:         #2D2D32
  text-primary:    #FAFAFA
  text-secondary: #8A8A8E
  accent:         #7C3AED
  recording:      #FF6B6B

Light Mode:
  background:     #FAFAFA
  surface:        #FFFFFF
  surface-hover:  #F5F5F5
  border:         #E5E5E7
  text-primary:    #0D0D0F
  text-secondary: #6B6B6E
  accent:         #6D28D9
  recording:      #DC2626
```

### Typography

- **Primary font:** SF Pro (macOS) / Segoe UI (Windows) — system fonts for native feel
- **Monospace:** SF Mono / Cascadia Code — for technical info (model name, latency)
- **Scale:** 11px (micro) / 13px (body) / 15px (subhead) / 20px (head) / 32px (hero)
- **Weight:** 400 (regular) / 500 (medium) / 600 (semibold)

### Spacing System

- Base unit: 4px
- Standard padding: 12px / 16px / 24px
- Component gaps: 8px / 12px
- Section gaps: 24px / 32px

### Motion

- **Instant feedback:** button presses, hotkey activation → < 50ms
- **Smooth transitions:** panel open/close → 200ms ease-out
- **Recording pulse:** pulsing red dot → 1.2s infinite ease-in-out
- **Waveform:** real-time audio level bars → 60fps canvas animation

---

## Component Inventory

### 1. Menu Bar Icon (NSStatusItem / TrayIcon)

**States:**
- **Idle:** Small mic icon, neutral gray (#8A8A8E)
- **Listening:** Animated waveform or pulsing red dot
- **Processing:** Spinning loader, violet (#7C3AED)
- **Success:** Brief green checkmark flash (200ms)
- **Error:** Brief red exclamation flash (200ms), then return to idle

### 2. Recording Indicator

Appears as a floating pill near the menu bar icon when recording is active.

```
● REC   00:03
```
- Red dot (pulsing) + elapsed time
- Press hotkey again or Esc to stop
- Tap anywhere to cancel

### 3. Settings Panel

Slide-out panel from menu bar, 320px wide.

**Sections:**
1. **Hotkey** — Current hotkey display, click to record new one
2. **Language** — Dropdown: Auto-detect + 10 major languages
3. **Polish Level** — Segmented control: Light / Medium / Strong
4. **LLM Provider** — Dropdown: Ollama (local) / OpenAI API
5. **Model Size** — Dropdown: Tiny / Base / Small / Medium
6. **About** — Version, open source link, check for updates

### 4. History Panel

Last 20 transcriptions shown in a scrollable list.

Each entry shows:
- Timestamp
- Original transcription (collapsed by default)
- Polished result (shown by default, one line preview)
- "Copy" / "Retry" / "Delete" actions

### 5. Status Bar (Settings Panel Footer)

Shows current system status:
- `○ Mic: ready` / `✗ Mic: unavailable`
- `○ LLM: Ollama (local)` / `○ LLM: OpenAI`
- Latency of last transcription: `⏱ 312ms`

---

## Interaction Flows

### First Run Onboarding

1. Welcome screen explains what Voix does (3 slides max)
2. Request microphone permission → system dialog
3. Request accessibility permission (for hotkey + text injection) → system dialog
4. Ask to download Whisper model (recommend "small" for balance)
5. Ask to configure LLM (Ollama or API key)
6. Done — show hotkey and "speak now" prompt

### Typical Session

1. User presses Option+E anywhere in any app
2. Menu bar icon turns red, recording indicator appears
3. User speaks (0.5s–30s)
4. User presses Option+E again (or Esc) to stop
5. Icon turns violet → "Polishing..."
6. Polished text appears in target app at cursor
7. Brief success animation on icon

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Microphone unavailable | Red icon, toast: "Microphone not found. Check permissions." |
| No internet + Ollama not running | Offer to use cloud API or show Ollama setup guide |
| Ollama not running | "Ollama not found. Start it or switch to API." + button to open Ollama |
| Text injection fails | Copy to clipboard, toast: "Couldn't inject. Copied to clipboard." |
| Whisper fails | Toast with error, offer to retry |

---

## Accessibility

- Full keyboard navigation (Tab through settings)
- VoiceOver / Narrator support for settings panel
- High contrast mode support
- Respect system dark/light preference
- Reduce motion preference respected

---

## Copy & Tone

- **Microcopy:** Minimal, functional, never cutesy
  - ✅ "Microphone ready"
  - ❌ "Hey there! Your mic is all set! 🎉"
- **Error messages:** Direct, actionable
  - ✅ "Ollama not running. Start it or switch to API in Settings."
  - ❌ "Oops! Something went wrong 😅"
- **Recording label:** "REC" in monospace, uppercase
- **No emoji in UI** — only in specific onboarding/celebratory moments
