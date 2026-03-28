import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

type Status = "idle" | "recording" | "processing" | "done" | "error";

interface HistoryEntry {
  id: number;
  timestamp: string;
  original: string;
  polished: string;
  language: string;
  latency_ms: number;
}

function App() {
  const [status, setStatus] = useState<Status>("idle");
  const [micAvailable, setMicAvailable] = useState(false);
  const [llmAvailable, setLlmAvailable] = useState(false);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [lastResult, setLastResult] = useState<{ original: string; polished: string } | null>(null);
  const [errorMsg, setErrorMsg] = useState("");
  const [elapsed, setElapsed] = useState(0);
  const [audioLevel, setAudioLevel] = useState(0);
  const tickerRef = useRef<number | null>(null);
  const levelRef = useRef<number | null>(null);
  const startTimeRef = useRef<number>(0);

  // Load initial state
  useEffect(() => {
    const init = async () => {
      try {
        await invoke("init_whisper");
        await invoke("init_llm");
        const mic = await invoke<boolean>("check_mic");
        setMicAvailable(mic);
        const llm = await invoke<boolean>("check_llm");
        setLlmAvailable(llm);
        const h = await invoke<HistoryEntry[]>("get_history", { limit: 20 });
        setHistory(h);
      } catch (e) {
        console.error("Init error:", e);
      }
    };
    init();
  }, []);

  const startTicker = useCallback(() => {
    startTimeRef.current = Date.now();
    tickerRef.current = window.setInterval(() => {
      setElapsed(Math.floor((Date.now() - startTimeRef.current) / 1000));
    }, 100);
    levelRef.current = window.setInterval(async () => {
      try {
        const level = await invoke<number>("get_audio_level");
        setAudioLevel(level);
      } catch {}
    }, 50);
  }, []);

  const stopTicker = useCallback(() => {
    if (tickerRef.current) clearInterval(tickerRef.current);
    if (levelRef.current) clearInterval(levelRef.current);
    setElapsed(0);
    setAudioLevel(0);
  }, []);

  const toggleRecording = useCallback(async () => {
    if (status === "idle") {
      try {
        await invoke("start_recording");
        setStatus("recording");
        startTicker();
      } catch (e: any) {
        setErrorMsg(e.toString());
        setStatus("error");
        setTimeout(() => setStatus("idle"), 3000);
      }
    } else if (status === "recording") {
      stopTicker();
      setStatus("processing");
      try {
        const result = await invoke<{ original: string; polished: string; latency_ms: number }>(
          "stop_and_process"
        );
        setLastResult({ original: result.original, polished: result.polished });
        await invoke("inject", { result });
        setStatus("done");
        setTimeout(() => setStatus("idle"), 3000);
        const h = await invoke<HistoryEntry[]>("get_history", { limit: 20 });
        setHistory(h);
      } catch (e: any) {
        setErrorMsg(e.toString());
        setStatus("error");
        setTimeout(() => setStatus("idle"), 3000);
      }
    }
  }, [status, startTicker, stopTicker]);

  // Listen for backend events
  useEffect(() => {
    const unlistenHotkey = listen("hotkey-triggered", () => {
      toggleRecording();
    });

    const unlistenStarted = listen("recording-started", () => {
      setStatus("recording");
      startTimeRef.current = Date.now();
    });

    const unlistenStopped = listen<string>("recording-stopped", (event) => {
      setLastResult({ original: "", polished: event.payload });
      setStatus("done");
      setTimeout(() => setStatus("idle"), 3000);
    });

    return () => {
      unlistenHotkey.then((f) => f());
      unlistenStarted.then((f) => f());
      unlistenStopped.then((f) => f());
    };
  }, [toggleRecording]);

  // Escape key to stop recording
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && status === "recording") {
        toggleRecording();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [status, toggleRecording]);

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">🎙️ Voix</h1>
        <span className="tagline">Voice → Polished Text</span>
      </header>

      <main className="main">
        {/* Status indicator */}
        <div className={`status-panel ${status}`}>
          {status === "idle" && (
            <button className="record-btn" onClick={toggleRecording} disabled={!micAvailable}>
              <span className="mic-icon">🎤</span>
              <span>Click or press Option+E</span>
            </button>
          )}

          {status === "recording" && (
            <div className="recording-ui">
              <div className="pulse-ring">
                <span className="rec-dot">●</span>
              </div>
              {/* Audio level bar */}
              <div className="level-bar">
                <div className="level-fill" style={{ width: `${audioLevel * 100}%` }} />
              </div>
              <span className="rec-label">REC</span>
              <span className="rec-time">{elapsed}s</span>
              <button className="stop-btn" onClick={toggleRecording}>■ Stop</button>
            </div>
          )}

          {status === "processing" && (
            <div className="processing-ui">
              <span className="spinner">◌</span>
              <span>Transcribing & Polishing...</span>
            </div>
          )}

          {status === "done" && (
            <div className="done-ui">
              <span className="check">✓</span>
              <span>Injected!</span>
            </div>
          )}

          {status === "error" && (
            <div className="error-ui">
              <span className="err-icon">✗</span>
              <span>{errorMsg || "Error"}</span>
            </div>
          )}
        </div>

        {/* Last result */}
        {lastResult && (
          <div className="last-result">
            {lastResult.original && (
              <div className="result-original">
                <span className="label">原始</span>
                <p>{lastResult.original}</p>
              </div>
            )}
            <div className="result-polished">
              <span className="label">润色后</span>
              <p>{lastResult.polished}</p>
            </div>
          </div>
        )}

        {/* History */}
        {history.length > 0 && (
          <div className="history">
            <h2>历史记录</h2>
            {history.slice(0, 5).map((entry) => (
              <div key={entry.id} className="history-entry">
                <div className="history-meta">
                  <span>{new Date(entry.timestamp).toLocaleTimeString()}</span>
                  <span>{entry.latency_ms}ms</span>
                </div>
                <div className="history-polished">{entry.polished}</div>
              </div>
            ))}
          </div>
        )}
      </main>

      <footer className="footer">
        <div className="status-bar">
          <span className={`mic-status ${micAvailable ? "ready" : "unavailable"}`}>
            {micAvailable ? "● Mic" : "✗ Mic"}
          </span>
          <span className={`llm-status ${llmAvailable ? "ready" : "offline"}`}>
            {llmAvailable ? "● LLM" : "○ LLM"}
          </span>
        </div>
      </footer>
    </div>
  );
}

export default App;
