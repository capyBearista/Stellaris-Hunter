import { useEffect, useState } from 'react';

import {
  loadAchievements,
  loadCatalogInfo,
  loadRuns,
  rescanSaves,
  type CatalogInfo,
  type PersistedRunSummary,
} from '../tauri';

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

export function Overview() {
  const [runs, setRuns] = useState<PersistedRunSummary[]>([]);
  const [catalogInfo, setCatalogInfo] = useState<CatalogInfo | null>(null);
  const [achievementCount, setAchievementCount] = useState<number>(0);
  const [status, setStatus] = useState<LoadState>('idle');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void loadPersistedOverview();
  }, []);

  const handleRescan = async () => {
    setStatus('loading');
    setError(null);

    try {
      setRuns(await rescanSaves());
      setStatus('ready');
    } catch (unknownError) {
      setError(errorMessage(unknownError));
      setStatus('error');
    }
  };

  async function loadPersistedOverview() {
    setStatus('loading');
    setError(null);

    try {
      const [loadedRuns, loadedCatalogInfo, achievements] = await Promise.all([
        loadRuns(),
        loadCatalogInfo(),
        loadAchievements(),
      ]);
      setRuns(loadedRuns);
      setCatalogInfo(loadedCatalogInfo);
      setAchievementCount(achievements.length);
      setStatus('ready');
    } catch (unknownError) {
      setError(errorMessage(unknownError));
      setStatus('error');
    }
  }

  const parsedRuns = runs.filter((run) => run.parse_status === 'parsed').length;
  const failedRuns = runs.filter((run) => run.parse_status === 'failed').length;
  const totalFacts = runs.reduce((sum, run) => sum + run.fact_count, 0);

  return (
    <>
      <header className="hero">
        <p className="eyebrow">Stellaris Hunter</p>
        <h1>Stellaris Hunter</h1>
        <p className="subtitle">Read-only local scan shell for install, documents, and save state.</p>
      </header>

      <section aria-labelledby="overview-heading" className="panel">
        <div className="panel-header">
          <h2 id="overview-heading">Overview</h2>
          <span className={`status status-${status}`}>{status}</span>
        </div>
        <dl className="summary-grid">
          <div>
            <dt>Catalog version</dt>
            <dd>{catalogInfo?.catalog_version ?? 'Unknown'}</dd>
          </div>
          <div>
            <dt>Achievements</dt>
            <dd>{achievementCount}</dd>
          </div>
          <div>
            <dt>Persisted runs</dt>
            <dd>{runs.length}</dd>
          </div>
          <div>
            <dt>Parsed runs</dt>
            <dd>{parsedRuns}</dd>
          </div>
          <div>
            <dt>Failed parses</dt>
            <dd>{failedRuns}</dd>
          </div>
          <div>
            <dt>Stored facts</dt>
            <dd>{totalFacts}</dd>
          </div>
        </dl>
      </section>

      <section aria-labelledby="scan-heading" className="panel">
        <div className="panel-header">
          <div>
            <h2 id="scan-heading">Save Scan</h2>
            <p className="muted panel-subtitle">Rescans saves and updates persisted run facts.</p>
          </div>
          <button type="button" onClick={handleRescan} disabled={status === 'loading'}>
            {status === 'loading' ? 'Scanning…' : 'Rescan Saves'}
          </button>
        </div>

        {error ? (
          <p role="alert" className="error">
            {error}
          </p>
        ) : null}
      </section>

      <section aria-labelledby="runs-heading" className="panel">
        <h2 id="runs-heading">Recent Runs</h2>
        {runs.length > 0 ? (
          <ul className="run-list">
            {runs.slice(0, 5).map((run) => (
              <li key={run.folder_path} className="run-list-item">
                <div className="run-list-main">
                  <strong>{run.display_name ?? run.run_folder}</strong>
                  <span>{run.latest_save_file_name ?? 'No latest save'}</span>
                </div>
                <div className="run-list-meta">
                  <span className={statusBadgeClass(run.parse_status)}>{run.parse_status ?? 'unknown'}</span>
                  {run.latest_ingame_date ? <span>{run.latest_ingame_date}</span> : null}
                  <span>{run.fact_count} facts</span>
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <p className="muted">No persisted runs yet. Use Rescan Saves to populate this summary.</p>
        )}
      </section>
    </>
  );
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

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
