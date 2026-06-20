import { useEffect, useState } from 'react';

import {
  loadAchievements,
  loadCatalogInfo,
  loadRuns,
  rescanSaves,
  type CatalogInfo,
  type PersistedRunSummary,
  type ScanReport,
} from '../tauri';
import { getCachedScanReport, scanLocalStateCached } from '../scanCache';

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

export function Overview() {
  const [runs, setRuns] = useState<PersistedRunSummary[]>([]);
  const [catalogInfo, setCatalogInfo] = useState<CatalogInfo | null>(null);
  const [achievementCount, setAchievementCount] = useState<number>(0);
  const [scanReport, setScanReport] = useState<ScanReport | null>(null);
  const [status, setStatus] = useState<LoadState>('idle');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void loadPersistedOverview();
  }, []);

  const handleRescan = async () => {
    setStatus('loading');
    setError(null);

    try {
      const [rescannedRuns, latestScanReport] = await Promise.all([
        rescanSaves(),
        scanLocalStateCached({ force: true }),
      ]);
      setRuns(rescannedRuns);
      setScanReport(latestScanReport);
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
      const existing = getCachedScanReport();
      const [loadedRuns, loadedCatalogInfo, achievements, latestScanReport] = await Promise.all([
        loadRuns(),
        loadCatalogInfo(),
        loadAchievements(),
        existing ? Promise.resolve(existing) : scanLocalStateCached(),
      ]);
      setRuns(loadedRuns);
      setCatalogInfo(loadedCatalogInfo);
      setAchievementCount(achievements.length);
      setScanReport(latestScanReport);
      setStatus('ready');
    } catch (unknownError) {
      setError(errorMessage(unknownError));
      setStatus('error');
    }
  }

  const parsedRuns = runs.filter((run) => run.parse_status === 'parsed').length;
  const failedRuns = runs.filter((run) => run.parse_status === 'failed').length;
  const totalFacts = runs.reduce((sum, run) => sum + run.fact_count, 0);
  const launcherDlcs = scanReport?.documents?.launcher?.dlcs ?? [];
  const launcherIssues = scanReport?.documents?.launcher?.issues ?? [];
  const documentIssues = scanReport?.documents?.issues ?? [];
  const scanErrors = scanReport?.errors ?? [];
  const documentsRoot = scanReport?.documents?.root;
  const disabledDlcCount = launcherDlcs.filter((dlc) => dlc.enabled_in_active_playset === false).length;
  const enabledDlcCount = launcherDlcs.filter((dlc) => dlc.enabled_in_active_playset === true).length;
  const dlcStatusLabel =
    launcherDlcs.length === 0
      ? 'Unknown'
      : disabledDlcCount === 0
        ? 'All enabled'
        : `${disabledDlcCount} disabled`;
  const dlcStatusDetail =
    launcherDlcs.length > 0
      ? `${enabledDlcCount} enabled locally · ${disabledDlcCount} disabled in the active playset.`
      : documentIssues[0]
        ? `Documents scan issue: ${documentIssues[0]}`
        : !scanReport?.documents
          ? 'Live scan could not locate the Stellaris Documents folder. Check Settings path overrides if your Documents folder is redirected.'
          : !scanReport?.documents?.launcher
            ? `Launcher database not found under ${documentsRoot ?? 'the detected Documents folder'}. If Documents is redirected, set the path override in Settings.`
            : launcherIssues[0]
              ? `Launcher scan issue: ${launcherIssues[0]}`
              : scanErrors[0]
                ? `Live scan error: ${scanErrors[0]}`
                : `No launcher DLC rows were discovered under ${documentsRoot ?? 'the detected Documents folder'}.`;

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
          <div>
            <dt>DLC status</dt>
            <dd>{dlcStatusLabel}</dd>
          </div>
        </dl>
        <p className="muted panel-subtitle">{dlcStatusDetail}</p>
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
