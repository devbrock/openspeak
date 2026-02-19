import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  downloadModel,
  getConfig,
  getStatus,
  setHotkey,
  setModel,
  setPasteMode,
  toggleRecording
} from './lib/tauri';
import type { AppConfig, AppStatus, TranscriptionResult } from './lib/types';

const EMPTY_STATUS: AppStatus = {
  recordingState: 'idle',
  modelReady: false,
  microphoneGranted: false,
  accessibilityGranted: false,
  lastError: null
};

const MODEL_OPTIONS = [
  { id: 'tiny', label: 'Tiny', detail: '75 MB, fastest' },
  { id: 'base', label: 'Base', detail: '142 MB' },
  { id: 'small', label: 'Small (Recommended)', detail: '466 MB' },
  { id: 'medium', label: 'Medium', detail: '1.5 GB' },
  { id: 'large-v3', label: 'Large-v3', detail: '3.1 GB, highest accuracy' },
  { id: 'turbo', label: 'Turbo', detail: 'Fast large model' }
] as const;
const PASTE_OPTIONS = ['clipboard', 'auto-paste'] as const;

export function App() {
  const [status, setStatus] = useState<AppStatus>(EMPTY_STATUS);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [result, setResult] = useState<TranscriptionResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hotkeyDraft, setHotkeyDraft] = useState('');
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [elapsedSec, setElapsedSec] = useState(0);

  const permissionSummary = useMemo(() => {
    if (!status.microphoneGranted) return 'Microphone permission required';
    if (!status.accessibilityGranted) return 'Accessibility permission required for paste automation';
    return 'Ready';
  }, [status.accessibilityGranted, status.microphoneGranted]);

  const recordingNow = status.recordingState === 'recording';

  const refresh = useCallback(async () => {
    const [nextStatus, nextConfig] = await Promise.all([getStatus(), getConfig()]);
    setStatus(nextStatus);
    setConfig(nextConfig);
    setHotkeyDraft((prev) => (prev ? prev : nextConfig.hotkey));
  }, []);

  useEffect(() => {
    refresh().catch((e: unknown) => {
      setError(e instanceof Error ? e.message : String(e));
    });
  }, [refresh]);

  useEffect(() => {
    const interval = window.setInterval(() => {
      void refresh();
    }, 1000);
    return () => window.clearInterval(interval);
  }, [refresh]);

  useEffect(() => {
    if (!recordingNow) {
      setStartedAt(null);
      setElapsedSec(0);
      return;
    }

    const base = startedAt ?? Date.now();
    if (!startedAt) setStartedAt(base);
    setElapsedSec(Math.max(0, Math.floor((Date.now() - base) / 1000)));

    const interval = window.setInterval(() => {
      setElapsedSec(Math.max(0, Math.floor((Date.now() - base) / 1000)));
    }, 500);
    return () => window.clearInterval(interval);
  }, [recordingNow, startedAt]);

  const onToggle = useCallback(async () => {
    setError(null);
    setBusy(true);
    try {
      const transcription = await toggleRecording();
      if (transcription) {
        setResult(transcription);
      } else {
        setResult(null);
      }
      await refresh();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [refresh]);

  const onChangeModel = useCallback(
    async (model: AppConfig['modelDefault']) => {
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

  const onSaveHotkey = useCallback(async () => {
    if (!hotkeyDraft.trim()) return;
    setError(null);
    setBusy(true);
    try {
      await setHotkey(hotkeyDraft.trim());
      await refresh();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [hotkeyDraft, refresh]);

  const onChangePasteMode = useCallback(
    async (mode: (typeof PASTE_OPTIONS)[number]) => {
      setError(null);
      setBusy(true);
      try {
        await setPasteMode(mode);
        await refresh();
      } catch (e: unknown) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setBusy(false);
      }
    },
    [refresh]
  );

  const statusError = error ?? status.lastError;
  const elapsedLabel = `${Math.floor(elapsedSec / 60)
    .toString()
    .padStart(2, '0')}:${(elapsedSec % 60).toString().padStart(2, '0')}`;

  return (
    <main className="app-shell">
      <div className="bg-orb bg-orb-left" />
      <div className="bg-orb bg-orb-right" />

      <section className="card settings-panel">
        <header className="panel-header">
          <div>
            <p className="eyebrow">VOICE WORKFLOW</p>
            <h1>OpenSpeak</h1>
            <p className="subhead">Local-first desktop dictation powered by whisper.cpp</p>
          </div>
          <span className={`state-pill state-${status.recordingState}`}>{status.recordingState}</span>
        </header>

        <div className="status-grid">
          <article className="status-item">
            <span>Recording Time</span>
            <strong>{recordingNow ? elapsedLabel : '--:--'}</strong>
          </article>
          <article className="status-item">
            <span>Permissions</span>
            <strong>{permissionSummary}</strong>
          </article>
          <article className="status-item">
            <span>Model</span>
            <strong>{config?.modelDefault ?? 'unknown'}</strong>
          </article>
          <article className="status-item">
            <span>Hotkey</span>
            <strong>{config?.hotkey ?? 'unknown'}</strong>
          </article>
        </div>

        <div className="actions action-row">
          <button className="btn btn-primary" onClick={onToggle} disabled={busy}>
            {recordingNow ? 'Stop Dictation' : 'Start Dictation'}
          </button>
          <button className="btn btn-secondary" onClick={onDownload} disabled={busy || !config}>
            Download Current Model
          </button>
        </div>

        <div className="settings-grid">
          <div className="field-group">
            <label htmlFor="model">Model</label>
            <select
              id="model"
              value={config?.modelDefault ?? 'small'}
              onChange={(e) => {
                const model = e.target.value as AppConfig['modelDefault'];
                void onChangeModel(model);
              }}
              disabled={busy}
            >
              {MODEL_OPTIONS.map((model) => (
                <option key={model.id} value={model.id}>
                  {model.label} - {model.detail}
                </option>
              ))}
            </select>
          </div>

          <div className="field-group">
            <label htmlFor="pasteMode">Output Mode</label>
            <select
              id="pasteMode"
              value={config?.pasteMode ?? 'clipboard'}
              onChange={(e) => {
                const mode = e.target.value as (typeof PASTE_OPTIONS)[number];
                void onChangePasteMode(mode);
              }}
              disabled={busy}
            >
              <option value="clipboard">Clipboard only</option>
              <option value="auto-paste">Auto-paste after stop</option>
            </select>
          </div>
        </div>

        <div className="field-group">
          <label htmlFor="hotkey">Global Hotkey</label>
          <div className="actions">
            <input
              id="hotkey"
              value={hotkeyDraft}
              onChange={(e) => setHotkeyDraft(e.target.value)}
              placeholder="CommandOrControl+Shift+Space"
              disabled={busy}
            />
            <button
              className="btn btn-secondary"
              onClick={() => void onSaveHotkey()}
              disabled={busy || !hotkeyDraft.trim()}
            >
              Save Hotkey
            </button>
          </div>
        </div>

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
              {result.delivery === 'auto-paste'
                ? 'Auto-paste was triggered into your active app.'
                : 'Copied to clipboard. Switch to any app and press '}
              {result.delivery === 'clipboard' ? (
                <>
                  <kbd>Cmd</kbd>+<kbd>V</kbd>.
                </>
              ) : null}
            </p>
          </article>
        ) : null}

        {statusError ? (
          <p className="error" role="status" aria-live="polite">
            {statusError}
          </p>
        ) : null}
      </section>
    </main>
  );
}
