export type RecordingState = 'idle' | 'recording' | 'transcribing';

export interface AppStatus {
  recordingState: RecordingState;
  modelReady: boolean;
  microphoneGranted: boolean;
  accessibilityGranted: boolean;
}

export interface TranscriptionResult {
  rawText: string;
  transformedText: string;
  commandsApplied: string[];
  latencyMs: number;
  confidence: number;
}

export interface AppConfig {
  hotkey: string;
  modelDefault: 'tiny' | 'base' | 'large';
  commandMode: 'basic';
  pasteMode: 'clipboard';
  language: 'en';
  privacy: {
    telemetryEnabled: boolean;
    persistAudioDebug: boolean;
  };
}
