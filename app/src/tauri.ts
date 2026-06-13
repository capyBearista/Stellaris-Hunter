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
  conditions: AchievementCondition[];
  warnings: string[];
  planner_notes: string | null;
  known_limitations: string[];
  rule_confidence: string | null;
}

export interface AchievementCondition {
  condition_type: string;
  dimension: string;
  operator: string;
  value: unknown;
  timing: string;
  mutability: string;
  severity: string;
  source: string | null;
  notes: string | null;
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

// --- Persisted runs/facts ---

export interface PersistedRunSummary {
  folder_path: string;
  run_folder: string;
  display_name: string | null;
  latest_save_path: string | null;
  latest_save_file_name: string | null;
  latest_ingame_date: string | null;
  game_version: string | null;
  parse_status: string | null;
  parse_error: string | null;
  fact_count: number;
  updated_at: string;
}

export interface RunFactSummary {
  run_folder_path: string;
  dimension: string;
  key: string;
  value: unknown;
  source: string;
  confidence: string;
  updated_from_save_path: string | null;
  updated_at: string;
}

// --- Planner/evaluation types ---

export interface ConditionEvaluation {
  dimension: string;
  operator: string;
  condition_value: unknown;
  fact_value: unknown | null;
  passed: boolean | null;
  severity: string;
  timing: string;
  mutability: string;
  reason: string;
}

export interface PlannerAchievementEvaluation {
  achievement: AchievementEntry;
  status: PlannerStatus;
  computed_status: PlannerStatus;
  planned: boolean;
  ignored: boolean;
  reasons: string[];
  warnings: string[];
  conditions: ConditionEvaluation[];
}

export type PlannerStatus =
  | 'Completed'
  | 'Planned'
  | 'Possible'
  | 'Incompatible'
  | 'Impossible'
  | 'Incomplete'
  | 'Unknown';

// --- IPC wrappers (existing) ---

export function scanLocalState() {
  return invoke<ScanReport>('scan_local_state', {});
}

export function loadRuns(): Promise<PersistedRunSummary[]> {
  return invoke<PersistedRunSummary[]>('load_runs', {});
}

export function loadRunFacts(runFolderPath: string): Promise<RunFactSummary[]> {
  return invoke<RunFactSummary[]>('load_run_facts', { runFolderPath });
}

export function rescanSaves(): Promise<PersistedRunSummary[]> {
  return invoke<PersistedRunSummary[]>('rescan_saves', {});
}

export function loadPlannerEvaluations(
  runFolderPath: string,
): Promise<PlannerAchievementEvaluation[]> {
  return invoke<PlannerAchievementEvaluation[]>('load_planner_evaluations', { runFolderPath });
}

export function setRunAchievementStatus(
  runFolderPath: string,
  achievementId: string,
  userStatus: 'planned' | 'ignored' | null,
): Promise<void> {
  return invoke<void>('set_run_achievement_status', { runFolderPath, achievementId, userStatus });
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
