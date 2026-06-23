import { useEffect, useMemo, useState } from 'react';

import {
  loadAchievements,
  loadRunAchievementNotes,
  loadRunNotes,
  loadRuns,
  type AchievementEntry,
  type PersistedRunSummary,
} from '../tauri';

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

interface RunWithNotes {
  run: PersistedRunSummary;
  runNote: string | null;
  achievementNotes: { achievementName: string; noteText: string }[];
}

export function Notes() {
  const [status, setStatus] = useState<LoadState>('idle');
  const [error, setError] = useState<string | null>(null);
  const [runsWithNotes, setRunsWithNotes] = useState<RunWithNotes[]>([]);

  useEffect(() => {
    let cancelled = false;
    setStatus('loading');
    setError(null);

    const loadAllNotes = async () => {
      try {
        const [runs, achievements] = await Promise.all([
          loadRuns(),
          loadAchievements(),
        ]);
        if (cancelled) return;

        const achievementMap = new Map<string, string>();
        for (const ach of achievements) {
          achievementMap.set(ach.id, ach.source.name);
        }

        const results = await Promise.all(
          runs.map(async (run) => {
            const [note, achNotes] = await Promise.all([
              loadRunNotes(run.folder_path),
              loadRunAchievementNotes(run.folder_path),
            ]);
            return {
              run,
              runNote: note?.note_text ?? null,
              achievementNotes: achNotes.map((an) => ({
                achievementName: achievementMap.get(an.achievement_id) ?? an.achievement_id,
                noteText: an.notes,
              })),
            };
          }),
        );

        if (!cancelled) {
          setRunsWithNotes(results);
          setStatus('ready');
        }
      } catch (unknownError) {
        if (!cancelled) {
          setError(errorMessage(unknownError));
          setStatus('error');
        }
      }
    };

    void loadAllNotes();
    return () => {
      cancelled = true;
    };
  }, []);

  const runsWithRunNotes = useMemo(
    () => runsWithNotes.filter((r) => r.runNote !== null && r.runNote.trim() !== ''),
    [runsWithNotes],
  );
  const runsWithAchievementNotes = useMemo(
    () => runsWithNotes.filter((r) => r.achievementNotes.length > 0),
    [runsWithNotes],
  );

  const hasAnyNotes = runsWithRunNotes.length > 0 || runsWithAchievementNotes.length > 0;

  return (
    <section aria-labelledby="notes-heading" className="panel">
      <div className="panel-header">
        <div>
          <h2 id="notes-heading">Notes</h2>
          <p className="muted panel-subtitle">Run notes and per-achievement notes across all campaigns.</p>
        </div>
      </div>

      {error ? <p role="alert" className="error">{error}</p> : null}

      {status === 'loading' ? (
        <p className="muted">Loading notes…</p>
      ) : status === 'ready' && !hasAnyNotes ? (
        <p className="muted">
          No notes found. Add notes to runs in the Runs page, or per-achievement notes in the Planner.
        </p>
      ) : status === 'ready' ? (
        <>
          {runsWithRunNotes.length > 0 ? (
            <section style={{ marginBottom: 'var(--space-24)' }}>
              <h3 style={{ marginBottom: 'var(--space-16)' }}>Run Notes</h3>
              {runsWithRunNotes.map((r) => (
                <div
                  key={r.run.folder_path}
                  className="run-fact-panel"
                  style={{ marginBottom: 'var(--space-16)' }}
                >
                  <h4>{r.run.display_name ?? r.run.run_folder}</h4>
                  <p className="muted" style={{ whiteSpace: 'pre-wrap' }}>{r.runNote}</p>
                </div>
              ))}
            </section>
          ) : null}

          {runsWithAchievementNotes.length > 0 ? (
            <section>
              <h3 style={{ marginBottom: 'var(--space-16)' }}>Achievement Notes</h3>
              {runsWithAchievementNotes.map((r) => (
                <div
                  key={r.run.folder_path}
                  className="run-fact-panel"
                  style={{ marginBottom: 'var(--space-16)' }}
                >
                  <h4>{r.run.display_name ?? r.run.run_folder}</h4>
                  <ul
                    className="planner-list"
                    style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))' }}
                  >
                    {r.achievementNotes.map((an) => (
                      <li key={an.achievementName} className="planner-item">
                        <strong>{an.achievementName}</strong>
                        <p className="muted" style={{ margin: 'var(--space-4) 0 0' }}>
                          {an.noteText}
                        </p>
                      </li>
                    ))}
                  </ul>
                </div>
              ))}
            </section>
          ) : null}
        </>
      ) : null}
    </section>
  );
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
