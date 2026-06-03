import { invoke } from '@tauri-apps/api/core';

// --- Achievement types ---

export interface AchievementSourceFields {
  name: string;
  description: string | null;
  requirement: string | null;
  hint: string | null;
  group: string | null;
  version_added: string | null;
  difficulty: string | null;
}

export interface AchievementCurationFields {
  tags: string[];
  conditions: unknown[];
  warnings: string[];
  planner_notes: string | null;
  known_limitations: string[];
  rule_confidence: string | null;
}

export interface AchievementEntry {
  id: string;
  steam_app_id: number;
  steam_api_name: string | null;
  local_key: string | null;
  deprecated: boolean;
  source: AchievementSourceFields;
  curation: AchievementCurationFields;
}

export interface CatalogInfo {
  catalog_version: string;
  stellaris_version: string | null;
  source_url: string | null;
  source_hash: string | null;
  updated_at: string;
  imported_at: string;
}

// --- Eligibility types ---

export interface SaveEligibility {
  conclusion: 'likely_eligible' | 'likely_ineligible' | 'unknown';
  cheated_on_save: boolean;
  ironman: boolean;
  mod_risk: 'none' | 'unknown' | 'checksum_scoped';
  reasons: string[];
  warnings: string[];
}

// --- ScanReport (existing, extended) ---

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
        ironman?: boolean | null;
        cheated_on_save?: boolean | null;
      } | null;
      eligibility?: SaveEligibility | null;
      issues?: string[];
    }> | null;
  } | null;
  errors?: string[];
}

// --- IPC wrappers (existing) ---

export function scanLocalState() {
  return invoke<ScanReport>('scan_local_state', {});
}

// --- IPC wrappers (new) ---

export function loadAchievements(): Promise<AchievementEntry[]> {
  return invoke<AchievementEntry[]>('load_achievements', {});
}

export function loadCatalogInfo(): Promise<CatalogInfo | null> {
  return invoke<CatalogInfo | null>('load_catalog_info', {});
}

// --- Achievement override types & wrappers ---

export interface AchievementOverride {
  achievement_id: string;
  completed: boolean;
}

export function loadCompletionOverrides(): Promise<AchievementOverride[]> {
  return invoke<AchievementOverride[]>('load_completion_overrides', {});
}

export function setCompletionOverride(achievementId: string, completed: boolean): Promise<void> {
  return invoke<void>('set_completion_override', { achievementId, completed });
}

export function clearCompletionOverride(achievementId: string): Promise<void> {
  return invoke<void>('clear_completion_override', { achievementId });
}
