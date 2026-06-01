import { invoke } from '@tauri-apps/api/core';

export interface ScanReport {
  install?: {
    version?: string | null;
    root?: string;
  } | null;
  documents?: {
    root?: string;
    save_runs?: Array<{
      run_folder: string;
      latest_save?: {
        name?: string | null;
        date?: string | null;
      } | null;
    }> | null;
  } | null;
  errors?: string[];
}

export function scanLocalState() {
  return invoke<ScanReport>('scan_local_state', {});
}
