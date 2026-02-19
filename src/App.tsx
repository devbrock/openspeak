import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  downloadModel,
  getConfig,
  getStatus,
  setModel,
  startRecording,
  stopRecording
} from './lib/tauri';
import type { AppConfig, AppStatus, TranscriptionResult } from './lib/types';

const EMPTY_STATUS: AppStatus = {
  recordingState: 'idle',
  modelReady: false,
  microphoneGranted: false,
  accessibilityGranted: false
};

const MODEL_OPTIONS = ['tiny', 'base', 'large'] as const;

export function App() {
  const [status, setStatus] = useState<AppStatus>(EMPTY_STATUS);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [result, setResult] = useState<TranscriptionResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const permissionSummary = useMemo(() => {
    if (!status.microphoneGranted) return 'Microphone permission required';
    if (!status.accessibilityGranted) return 'Accessibility permission required for paste automation';
    return 'Ready';
  }, [status.accessibilityGranted, status.microphoneGranted]);

  const refresh = useCallback(async () => {
    const [nextStatus, nextConfig] = await Promise.all([getStatus(), getConfig()]);
    setStatus(nextStatus);
    setConfig(nextConfig);
  }, []);

  useEffect(() => {
    refresh().catch((e: unknown) => {
      setError(e instanceof Error ? e.message : String(e));
    });
  }, [refresh]);

  const onToggle = useCallback(async () => {
    setError(null);
    setBusy(true);
    try {
      if (!sessionId) {
        const id = await startRecording();
        setSessionId(id);
        setResult(null);
      } else {
        const transcription = await stopRecording(sessionId);
        setSessionId(null);
        setResult(transcription);
      }
      await refresh();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [refresh, sessionId]);

  const onChangeModel = useCallback(
    async (model: (typeof MODEL_OPTIONS)[number]) => {
      setError(null);
      setBusy(true);
      try {
        await setModel(model);
        await refresh();
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setBusy(false);
      }
    },
    [refresh]
  );

  const onDownload = useCallback(async () => {
    if (!config) return;
    setError(null);
    setBusy(true);
    try {
      await downloadModel(config.modelDefault);
      await refresh();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [config, refresh]);

  return (
    <main className="app-shell">
      <section className="card">
        <h1>Brock's Dictation Tool</h1>
        <p className="subhead">macOS-first local dictation using whisper.cpp</p>

        <div className="status-grid">
          <span>State</span>
          <strong>{status.recordingState}</strong>
          <span>Permissions</span>
          <strong>{permissionSummary}</strong>
          <span>Model</span>
          <strong>{config?.modelDefault ?? 'unknown'}</strong>
        </div>

        <div className="actions">
          <button onClick={onToggle} disabled={busy}>
            {sessionId ? 'Stop Dictation' : 'Start Dictation'}
          </button>
          <button onClick={onDownload} disabled={busy || !config}>
            Download Current Model
          </button>
        </div>

        <label htmlFor="model">Model</label>
        <select
          id="model"
          value={config?.modelDefault ?? 'tiny'}
          onChange={(e) => {
            const model = e.target.value as (typeof MODEL_OPTIONS)[number];
            void onChangeModel(model);
          }}
          disabled={busy}
        >
          {MODEL_OPTIONS.map((m) => (
            <option key={m} value={m}>
              {m}
            </option>
          ))}
        </select>

        {result ? (
          <article className="result">
            <h2>Last Dictation</h2>
            <p>{result.transformedText}</p>
            <small>
              confidence={result.confidence.toFixed(2)}, latency={result.latencyMs}ms, commands={
                result.commandsApplied.length
              }
            </small>
            <p>
              Copied to clipboard. Switch to any app and press <kbd>Cmd</kbd>+<kbd>V</kbd>.
            </p>
          </article>
        ) : null}

        {error ? <p className="error">{error}</p> : null}
      </section>
    </main>
  );
}
