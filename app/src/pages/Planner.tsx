import { useEffect, useMemo, useState } from 'react';

import {
  clearRunAchievementNote,
  loadPlannerEvaluations,
  loadRunAchievementNotes,
  loadRuns,
  setRunAchievementNote,
  setRunAchievementStatus,
  type PersistedRunSummary,
  type PlannerAchievementEvaluation,
  type PlannerStatus,
} from '../tauri';

const STATUS_ORDER: PlannerStatus[] = [
  'Planned',
  'Possible',
  'Incompatible',
  'Impossible',
  'Unknown',
  'Completed',
  'Incomplete',
];

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

export function Planner() {
  const [runs, setRuns] = useState<PersistedRunSummary[]>([]);
  const [selectedRunPath, setSelectedRunPath] = useState<string | null>(null);
  const [evaluations, setEvaluations] = useState<PlannerAchievementEvaluation[]>([]);
  const [status, setStatus] = useState<LoadState>('idle');
  const [evalStatus, setEvalStatus] = useState<LoadState>('idle');
  const [achievementNotes, setAchievementNotes] = useState<Map<string, string>>(new Map());
  const [collapsedGroups, setCollapsedGroups] = useState<Set<PlannerStatus>>(new Set());
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setStatus('loading');
    loadRuns()
      .then((loadedRuns) => {
        if (!cancelled) {
          setRuns(loadedRuns);
          setSelectedRunPath(loadedRuns[0]?.folder_path ?? null);
          setStatus('ready');
        }
      })
      .catch((unknownError) => {
        if (!cancelled) {
          setError(errorMessage(unknownError));
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!selectedRunPath) {
      setEvaluations([]);
      setEvalStatus('idle');
      return;
    }

    let cancelled = false;
    setEvalStatus('loading');
    setError(null);
    loadPlannerEvaluations(selectedRunPath)
      .then((loadedEvaluations) => {
        if (!cancelled) {
          setEvaluations(loadedEvaluations);
          setEvalStatus('ready');
        }
      })
      .catch((unknownError) => {
        if (!cancelled) {
          setError(errorMessage(unknownError));
          setEvalStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [selectedRunPath]);

  useEffect(() => {
    if (!selectedRunPath) {
      setAchievementNotes(new Map());
      return;
    }
    let cancelled = false;
    loadRunAchievementNotes(selectedRunPath)
      .then((notes) => {
        if (!cancelled) {
          const map = new Map<string, string>();
          for (const note of notes) {
            map.set(note.achievement_id, note.notes);
          }
          setAchievementNotes(map);
        }
      })
      .catch(() => {
        // Non-critical — silently ignore
      });
    return () => { cancelled = true; };
  }, [selectedRunPath]);

  const grouped = useMemo(() => {
    const groups = new Map<PlannerStatus, PlannerAchievementEvaluation[]>();
    for (const status of STATUS_ORDER) groups.set(status, []);
    for (const evaluation of evaluations) {
      const bucket = groups.get(evaluation.status) ?? groups.get('Unknown');
      bucket?.push(evaluation);
    }
    return groups;
  }, [evaluations]);

  const selectedRun = runs.find((run) => run.folder_path === selectedRunPath) ?? null;

  const handlePlannedToggle = async (evaluation: PlannerAchievementEvaluation) => {
    if (!selectedRunPath) return;
    const nextStatus = evaluation.planned ? null : 'planned';
    try {
      await setRunAchievementStatus(selectedRunPath, evaluation.achievement.id, nextStatus);
      setEvaluations((current) =>
        current.map((item) => {
          if (item.achievement.id !== evaluation.achievement.id) return item;
          const planned = nextStatus === 'planned';
          return {
            ...item,
            planned,
            ignored: planned ? false : item.ignored,
            status: planned ? 'Planned' : item.computed_status,
          };
        }),
      );
    } catch (unknownError) {
      setError(errorMessage(unknownError));
    }
  };

  const toggleGroupCollapse = (status: PlannerStatus) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(status)) next.delete(status);
      else next.add(status);
      return next;
    });
  };

  const collapseAllGroups = () => {
    setCollapsedGroups(new Set(STATUS_ORDER));
  };

  const expandAllGroups = () => {
    setCollapsedGroups(new Set());
  };

  return (
    <section aria-labelledby="planner-heading" className="panel planner-panel">
      <div className="panel-header">
        <div>
          <h2 id="planner-heading">Planner</h2>
          <p className="muted panel-subtitle">
            Conservative achievement compatibility for the selected persisted run.
          </p>
        </div>
        <select
          className="filter-select"
          value={selectedRunPath ?? ''}
          onChange={(event) => setSelectedRunPath(event.target.value || null)}
          disabled={status === 'loading' || runs.length === 0}
          aria-label="Selected run"
        >
          {runs.length === 0 ? <option value="">No persisted runs</option> : null}
          {runs.map((run) => (
            <option key={run.folder_path} value={run.folder_path}>
              {run.display_name ?? run.run_folder}
            </option>
          ))}
        </select>
      </div>

      {error ? <p role="alert" className="error">{error}</p> : null}

      {!selectedRun ? (
        <p className="muted">
          {status === 'loading'
            ? 'Loading runs…'
            : 'No persisted runs yet. Use Runs / Saves to rescan saves first.'}
        </p>
      ) : (
        <>
          <p className="catalog-info muted">
            Evaluating {selectedRun.display_name ?? selectedRun.run_folder}
            {selectedRun.latest_ingame_date ? ` · ${selectedRun.latest_ingame_date}` : ''}
            {selectedRun.fact_count ? ` · ${selectedRun.fact_count} parsed facts` : ''}
          </p>
          {evalStatus === 'loading' ? <p className="muted">Evaluating achievements…</p> : null}
          {evalStatus !== 'loading' ? (
            <>
              <div className="planner-collapse-controls">
                <button type="button" className="secondary-button" onClick={collapseAllGroups}>
                  Collapse All
                </button>
                <button type="button" className="secondary-button" onClick={expandAllGroups}>
                  Expand All
                </button>
              </div>
              <div className="planner-groups">
                {STATUS_ORDER.map((statusName) => {
                  const items = grouped.get(statusName) ?? [];
                  if (items.length === 0) return null;
                  const isCollapsed = collapsedGroups.has(statusName);
                  return (
                    <section key={statusName} className="planner-group" aria-label={`${statusName} achievements`}>
                      <h3>
                        <button
                          type="button"
                          className="planner-group-toggle"
                          onClick={() => toggleGroupCollapse(statusName)}
                          aria-expanded={!isCollapsed}
                          aria-label={`${isCollapsed ? 'Expand' : 'Collapse'} ${statusName} group`}
                        >
                          {isCollapsed ? '\u25B6' : '\u25BC'}
                        </button>
                        <span className={statusBadgeClass(statusName)}>{statusName}</span>
                        <span className="muted">{items.length}</span>
                      </h3>
                      {!isCollapsed ? (
                        <>
                          <ul className="planner-list">
                            {items.slice(0, 25).map((evaluation) => (
                              <PlannerItem
                                key={evaluation.achievement.id}
                                evaluation={evaluation}
                                selectedRunPath={selectedRunPath}
                                noteText={achievementNotes.get(evaluation.achievement.id) ?? ''}
                                onPlannedToggle={() => void handlePlannedToggle(evaluation)}
                                onNoteChange={(achievementId, text) => {
                                  setAchievementNotes((prev) => {
                                    const next = new Map(prev);
                                    if (text) next.set(achievementId, text);
                                    else next.delete(achievementId);
                                    return next;
                                  });
                                }}
                              />
                            ))}
                          </ul>
                          {items.length > 25 ? (
                            <p className="muted">Showing first 25 of {items.length} in this group.</p>
                          ) : null}
                        </>
                      ) : null}
                    </section>
                  );
                })}
              </div>
            </>
          ) : null}
        </>
      )}
    </section>
  );
}

function PlannerItem({
  evaluation,
  selectedRunPath,
  noteText,
  onPlannedToggle,
  onNoteChange,
}: {
  evaluation: PlannerAchievementEvaluation;
  selectedRunPath: string | null;
  noteText: string;
  onPlannedToggle: () => void;
  onNoteChange: (achievementId: string, text: string) => void;
}) {
  const [showNotes, setShowNotes] = useState(false);
  const [showDetail, setShowDetail] = useState(false);
  const [editNote, setEditNote] = useState(noteText);
  const [noteError, setNoteError] = useState<string | null>(null);

  // Sync editNote when prop changes
  useEffect(() => {
    setEditNote(noteText);
  }, [noteText]);

  const handleSaveNote = async () => {
    if (!selectedRunPath) return;
    setNoteError(null);
    try {
      await setRunAchievementNote(selectedRunPath, evaluation.achievement.id, editNote);
      onNoteChange(evaluation.achievement.id, editNote);
    } catch (err) {
      setNoteError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleClearNote = async () => {
    if (!selectedRunPath) return;
    setNoteError(null);
    try {
      await clearRunAchievementNote(selectedRunPath, evaluation.achievement.id);
      setEditNote('');
      onNoteChange(evaluation.achievement.id, '');
    } catch (err) {
      setNoteError(err instanceof Error ? err.message : String(err));
    }
  };

  // DLC condition that is blocking or unknown
  const dlcCondition = evaluation.conditions.find((c) =>
    ['required_dlc', 'dlc_required', 'dlc'].includes(c.dimension),
  );
  const needsDlcAttention = dlcCondition && dlcCondition.passed !== true;
  const hasWarnings = evaluation.warnings.length > 0;

  // First failing (or unknown) condition for compact blocker display
  const blockerCondition = evaluation.conditions.find((c) => c.passed !== true);
  const detailReason = blockerCondition ? (evaluation.reasons[0] ?? null) : null;
  const showsAdvisoryRow = !blockerCondition && hasWarnings;

  return (
    <li className="planner-item" data-status={evaluation.status}>
      <div className="planner-item-main">
        <div>
          <strong>{evaluation.achievement.source.name}</strong>
          <p className="muted">{evaluation.achievement.source.requirement ?? 'No requirement text.'}</p>
        </div>
        <button type="button" className="secondary-button" onClick={onPlannedToggle}>
          {evaluation.planned ? 'Unplan' : 'Plan'}
        </button>
      </div>
      {blockerCondition ? (
        <div className="planner-blocker">
          <span className={needsDlcAttention ? 'planner-dlc-label' : 'planner-blocker-label'}>
            {needsDlcAttention
              ? (typeof dlcCondition!.condition_value === 'string'
                  ? `Missing: ${dlcCondition!.condition_value}`
                  : 'Missing required DLC')
              : (blockerCondition.reason || `${blockerCondition.dimension} not satisfied`)}
          </span>
          <button
            type="button"
            className="link-button"
            onClick={() => setShowDetail((v) => !v)}
          >
            {showDetail ? 'Hide details' : 'Details'}
          </button>
        </div>
      ) : null}
      {showsAdvisoryRow ? (
        <div className="planner-blocker planner-advisory">
          <span className="detail-warning">Note</span>
          <button
            type="button"
            className="link-button"
            onClick={() => setShowDetail((v) => !v)}
          >
            {showDetail ? 'Hide details' : 'Details'}
          </button>
        </div>
      ) : null}
      {showDetail && (detailReason || hasWarnings) ? (
        <div className="planner-detail-reason">
          {detailReason ? <p>{detailReason}</p> : null}
          {hasWarnings ? (
            evaluation.warnings.length === 1 ? (
              <p>{evaluation.warnings[0]}</p>
            ) : (
              <ul>
                {evaluation.warnings.map((warning) => (
                  <li key={warning}>{warning}</li>
                ))}
              </ul>
            )
          ) : null}
        </div>
      ) : null}
      <div className="planner-meta">
        {evaluation.achievement.source.difficulty ? (
          <span className="badge badge-medium">{evaluation.achievement.source.difficulty}</span>
        ) : null}
        {needsDlcAttention ? <span className="badge badge-dlc-warning">DLC attention</span> : null}
        {evaluation.achievement.curation.tags.slice(0, 4).map((tag) => (
          <span key={tag} className="tag-pill">{tag}</span>
        ))}
        {hasWarnings && !showsAdvisoryRow ? <span className="detail-warning">Note</span> : null}
      </div>
      <div className="planner-note-section">
        <button
          type="button"
          className="link-button"
          onClick={() => setShowNotes((v) => !v)}
        >
          {showNotes ? 'Hide notes' : 'Notes'}
          {noteText ? ' (has note)' : ''}
        </button>
        {showNotes ? (
          <div className="planner-note-form">
            <textarea
              className="filter-input"
              rows={2}
              value={editNote}
              onChange={(e) => setEditNote(e.target.value)}
              placeholder="Add a note for this achievement in this run…"
            />
            {noteError ? <p role="alert" className="error">{noteError}</p> : null}
            <div className="fact-edit-actions">
              <button type="button" onClick={handleSaveNote}>Save note</button>
              {noteText ? (
                <button type="button" className="secondary-button" onClick={handleClearNote}>
                  Clear
                </button>
              ) : null}
            </div>
          </div>
        ) : null}
      </div>
    </li>
  );
}

function statusBadgeClass(status: PlannerStatus) {
  if (status === 'Possible' || status === 'Completed' || status === 'Planned') {
    return 'badge badge-eligible';
  }
  if (status === 'Impossible' || status === 'Incompatible') {
    return 'badge badge-ineligible';
  }
  return 'badge badge-unknown';
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
