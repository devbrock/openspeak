import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, AppStatus, TranscriptionResult } from './types';

export async function getStatus(): Promise<AppStatus> {
  return invoke<AppStatus>('get_status');
}

export async function startRecording(): Promise<string> {
  return invoke<string>('start_recording');
}

export async function stopRecording(sessionId: string): Promise<TranscriptionResult> {
  return invoke<TranscriptionResult>('stop_recording', { sessionId });
}

export async function setHotkey(hotkeySpec: string): Promise<void> {
  return invoke('set_hotkey', { hotkeySpec });
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export async function setModel(modelId: 'tiny' | 'base' | 'large'): Promise<void> {
  return invoke('set_model', { modelId });
}

export async function downloadModel(modelId: 'tiny' | 'base' | 'large'): Promise<string> {
  return invoke<string>('download_model', { modelId });
}
