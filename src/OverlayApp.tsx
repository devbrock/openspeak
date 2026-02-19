import { useEffect, useMemo, useState } from 'react';
import { getStatus } from './lib/tauri';
import type { AppStatus } from './lib/types';

const EMPTY_STATUS: AppStatus = {
  recordingState: 'idle',
  modelReady: false,
  microphoneGranted: false,
  accessibilityGranted: false,
  lastError: null
};

const BAR_COUNT = 22;

export function OverlayApp() {
  const [status, setStatus] = useState<AppStatus>(EMPTY_STATUS);

  useEffect(() => {
    let mounted = true;

    async function tick() {
      try {
        const next = await getStatus();
        if (mounted) {
          setStatus(next);
        }
      } catch {
        // Keep overlay minimal and non-blocking.
      }
    }

    void tick();
    const interval = window.setInterval(() => {
      void tick();
    }, 200);

    return () => {
      mounted = false;
      window.clearInterval(interval);
    };
  }, []);

  const mode = status.recordingState;
  const label = useMemo(() => {
    if (mode === 'recording') return 'Recording';
    if (mode === 'transcribing') return 'Transcribing';
    return 'Idle';
  }, [mode]);

  const active = mode !== 'idle';

  return (
    <main className="overlay-shell" aria-live="polite">
      <section className={`overlay-card ${active ? 'overlay-active' : ''}`}>
        <div className="overlay-topline">
          <span className={`overlay-dot mode-${mode}`} />
          <strong>{label}</strong>
        </div>

        {mode === 'recording' ? (
          <div className="wave" role="img" aria-label="Recording waveform animation">
            {Array.from({ length: BAR_COUNT }).map((_, idx) => (
              <span
                key={idx}
                className="wave-bar"
                style={{ animationDelay: `${idx * 45}ms` }}
              />
            ))}
          </div>
        ) : null}

        {mode === 'transcribing' ? <div className="overlay-processing">Processing speech...</div> : null}
      </section>
    </main>
  );
}
