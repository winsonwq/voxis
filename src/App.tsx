import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
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

  useEffect(() => {
    loadState();
  }, []);

  const loadState = async () => {
    try {
      await invoke("get_settings");
      const mic = await invoke<boolean>("check_mic");
      setMicAvailable(mic);
      const llm = await invoke<boolean>("check_llm");
      setLlmAvailable(llm);
      const h = await invoke<HistoryEntry[]>("get_history", { limit: 20 });
      setHistory(h);
      await invoke("init_whisper");
      await invoke("init_llm");
    } catch (e) {
      console.error("Init error:", e);
    }
  };

  const startRecording = useCallback(async () => {
    if (status !== "idle") return;
    setStatus("recording");
    setErrorMsg("");
    setElapsed(0);
    const start = Date.now();
    const ticker = setInterval(() => setElapsed(Math.floor((Date.now() - start) / 1000)), 100);
    (window as any).__voxis_ticker = ticker;
  }, [status]);

  const stopRecording = useCallback(async () => {
    if (status !== "recording") return;
    const ticker = (window as any).__voxis_ticker;
    if (ticker) clearInterval(ticker);
    setStatus("processing");

    try {
      const result = await invoke<{ original: string; polished: string; latency_ms: number }>(
        "transcribe_and_polish",
        { durationSecs: Math.max(1, elapsed) }
      );
      setLastResult({ original: result.original, polished: result.polished });
      await invoke("inject", { result });
      setStatus("done");
      setTimeout(() => setStatus("idle"), 2000);
      const h = await invoke<HistoryEntry[]>("get_history", { limit: 20 });
      setHistory(h);
    } catch (e: any) {
      setErrorMsg(e.toString());
      setStatus("error");
      setTimeout(() => setStatus("idle"), 3000);
    }
  }, [status, elapsed]);

  // Keyboard shortcut
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape" && status === "recording") {
        stopRecording();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [status, stopRecording]);

  const micReady = micAvailable ? "ready" : "unavailable";
  const llmReady = llmAvailable ? "ready" : "offline";

  return (
    <div className="app">
      <header className="header">
        <h1 className="logo">Voix</h1>
        <span className="tagline">Voice → Polished Text</span>
      </header>

      <main className="main">
        {/* Status indicator */}
        <div className={`status-panel ${status}`}>
          {status === "idle" && (
            <button className="record-btn" onClick={startRecording} disabled={!micAvailable}>
              <span className="mic-icon">🎙️</span>
              <span>Click or press Option+E</span>
            </button>
          )}
          {status === "recording" && (
            <div className="recording-ui">
              <div className="pulse-ring">
                <span className="rec-dot">●</span>
              </div>
              <span className="rec-label">REC</span>
              <span className="rec-time">{elapsed}s</span>
              <button className="stop-btn" onClick={stopRecording}>Stop</button>
            </div>
          )}
          {status === "processing" && (
            <div className="processing-ui">
              <span className="spinner">◌</span>
              <span>Polishing...</span>
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
        {lastResult && status === "done" && (
          <div className="last-result">
            <div className="result-original">{lastResult.original}</div>
            <div className="result-polished">{lastResult.polished}</div>
          </div>
        )}

        {/* History */}
        {history.length > 0 && (
          <div className="history">
            <h2>History</h2>
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
          <span className={`mic-status ${micReady}`}>
            {micAvailable ? "● Mic ready" : "✗ Mic unavailable"}
          </span>
          <span className={`llm-status ${llmReady}`}>
            {llmAvailable ? "● LLM ready" : "○ LLM offline"}
          </span>
        </div>
        <button className="settings-btn" onClick={() => invoke("open_settings")}>
          Settings
        </button>
      </footer>
    </div>
  );
}

export default App;
