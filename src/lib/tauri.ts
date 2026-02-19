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

export async function toggleRecording(): Promise<TranscriptionResult | null> {
  return invoke<TranscriptionResult | null>('toggle_recording');
}

export async function setHotkey(hotkeySpec: string): Promise<void> {
  return invoke('set_hotkey', { hotkeySpec });
}

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export async function resetPermissions(): Promise<void> {
  return invoke('reset_permissions');
}

export async function enablePermissions(): Promise<void> {
  return invoke('enable_permissions');
}

export async function setModel(
  modelId: 'tiny' | 'base' | 'small' | 'medium' | 'large-v3' | 'turbo'
): Promise<void> {
  return invoke('set_model', { modelId });
}

export async function setPasteMode(pasteMode: 'clipboard' | 'auto-paste'): Promise<void> {
  return invoke('set_paste_mode', { pasteMode });
}

export async function downloadModel(
  modelId: 'tiny' | 'base' | 'small' | 'medium' | 'large-v3' | 'turbo'
): Promise<string> {
  return invoke<string>('download_model', { modelId });
}
