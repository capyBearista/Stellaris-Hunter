import { useEffect, useState } from 'react';

import {
  loadRunFacts,
  loadRuns,
  rescanSaves,
  type PersistedRunSummary,
  type RunFactSummary,
} from '../tauri';

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

export function Runs() {
  const [runs, setRuns] = useState<PersistedRunSummary[]>([]);
  const [selectedRunPath, setSelectedRunPath] = useState<string | null>(null);
  const [facts, setFacts] = useState<RunFactSummary[]>([]);
  const [status, setStatus] = useState<LoadState>('idle');
  const [factStatus, setFactStatus] = useState<LoadState>('idle');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refreshRuns(loadRuns, 'loading');
  }, []);

  useEffect(() => {
    if (!selectedRunPath) {
      setFacts([]);
      return;
    }

    let cancelled = false;
    setFactStatus('loading');
    loadRunFacts(selectedRunPath)
      .then((loadedFacts) => {
        if (!cancelled) {
          setFacts(loadedFacts);
          setFactStatus('ready');
        }
      })
      .catch((unknownError) => {
        if (!cancelled) {
          setError(errorMessage(unknownError));
          setFactStatus('error');
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedRunPath]);

  const handleRescan = async () => {
    await refreshRuns(rescanSaves, 'loading');
  };

  async function refreshRuns(
    loader: () => Promise<PersistedRunSummary[]>,
    loadingState: LoadState,
  ) {
    setStatus(loadingState);
    setError(null);

    try {
      const loadedRuns = await loader();
      setRuns(loadedRuns);
      setSelectedRunPath((current) => keepExistingSelection(current, loadedRuns));
      setStatus('ready');
    } catch (unknownError) {
      setError(errorMessage(unknownError));
      setStatus('error');
    }
  }

  const selectedRun = runs.find((run) => run.folder_path === selectedRunPath) ?? null;

  return (
    <section aria-labelledby="runs-heading" className="panel">
      <div className="panel-header">
        <div>
          <h2 id="runs-heading">Runs / Saves</h2>
          <p className="muted panel-subtitle">Persisted save folders and latest parsed facts.</p>
        </div>
        <button type="button" onClick={handleRescan} disabled={status === 'loading'}>
          {status === 'loading' ? 'Scanning…' : 'Rescan Saves'}
        </button>
      </div>

      {error ? <p role="alert" className="error">{error}</p> : null}

      {runs.length > 0 ? (
        <div className="runs-layout">
          <ul className="run-list" aria-label="Persisted runs">
            {runs.map((run) => (
              <li key={run.folder_path} className="run-list-item">
                <button
                  type="button"
                  className="run-select-button"
                  onClick={() => setSelectedRunPath(run.folder_path)}
                  aria-pressed={run.folder_path === selectedRunPath}
                >
                  <span className="run-list-main">
                    <strong>{run.display_name ?? run.run_folder}</strong>
                    <span>{run.latest_save_file_name ?? 'No latest save'}</span>
                  </span>
                  <span className="run-list-meta">
                    <span className={statusBadgeClass(run.parse_status)}>
                      {run.parse_status ?? 'unknown'}
                    </span>
                    {run.latest_ingame_date ? <span>{run.latest_ingame_date}</span> : null}
                    {run.game_version ? <span>{run.game_version}</span> : null}
                    <span>{run.fact_count} facts</span>
                  </span>
                </button>
                {run.parse_error ? <p className="error run-error">{run.parse_error}</p> : null}
              </li>
            ))}
          </ul>

          <RunFactPanel run={selectedRun} facts={facts} factStatus={factStatus} />
        </div>
      ) : (
        <p className="muted">
          {status === 'loading' ? 'Loading persisted runs…' : 'No persisted runs yet. Rescan saves to populate this view.'}
        </p>
      )}
    </section>
  );
}

function RunFactPanel({
  run,
  facts,
  factStatus,
}: {
  run: PersistedRunSummary | null;
  facts: RunFactSummary[];
  factStatus: LoadState;
}) {
  const [showAllFacts, setShowAllFacts] = useState(false);

  if (!run) {
    return <aside className="run-fact-panel muted">Select a run to view parsed facts.</aside>;
  }

  const visibleFacts = showAllFacts ? facts : facts.slice(0, 12);

  return (
    <aside className="run-fact-panel" aria-label="Selected run facts">
      <h3>{run.display_name ?? run.run_folder}</h3>
      <dl className="run-detail-list">
        <dt>Folder</dt>
        <dd>{run.folder_path}</dd>
        <dt>Latest save</dt>
        <dd>{run.latest_save_path ?? 'Unknown'}</dd>
      </dl>

      <h4>Parsed facts</h4>
      {factStatus === 'loading' ? <p className="muted">Loading facts…</p> : null}
      {facts.length > 0 ? (
        <ul className="fact-list">
          {visibleFacts.map((fact) => (
            <li key={`${fact.dimension}:${fact.key}`}>
              <span className="fact-key">{fact.dimension}.{fact.key}</span>
              <span className="fact-value">{formatFactValue(fact.value)}</span>
              <span className="muted">{fact.confidence}</span>
            </li>
          ))}
        </ul>
      ) : factStatus !== 'loading' ? (
        <p className="muted">No parsed facts stored for this run.</p>
      ) : null}
      {facts.length > 12 ? (
        <button
          type="button"
          className="link-button"
          onClick={() => setShowAllFacts((current) => !current)}
        >
          {showAllFacts ? 'Show fewer facts' : `Show all ${facts.length} facts`}
        </button>
      ) : null}
    </aside>
  );
}

function keepExistingSelection(current: string | null, runs: PersistedRunSummary[]) {
  if (current && runs.some((run) => run.folder_path === current)) {
    return current;
  }
  return runs[0]?.folder_path ?? null;
}

function statusBadgeClass(status: string | null) {
  if (status === 'parsed') {
    return 'badge badge-eligible';
  }
  if (status === 'failed') {
    return 'badge badge-ineligible';
  }
  return 'badge badge-unknown';
}

function formatFactValue(value: unknown) {
  if (Array.isArray(value)) {
    return value.join(', ');
  }
  if (typeof value === 'boolean') {
    return value ? 'yes' : 'no';
  }
  if (value === null || value === undefined) {
    return 'Unknown';
  }
  return String(value);
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
