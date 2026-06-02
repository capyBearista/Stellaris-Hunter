import { useState } from 'react';

import { scanLocalState, type ScanReport } from '../tauri';

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

export function Overview() {
  const [report, setReport] = useState<ScanReport | null>(null);
  const [status, setStatus] = useState<LoadState>('idle');
  const [error, setError] = useState<string | null>(null);

  const handleScan = async () => {
    setStatus('loading');
    setError(null);

    try {
      setReport(await scanLocalState());
      setStatus('ready');
    } catch (unknownError) {
      setError(unknownError instanceof Error ? unknownError.message : String(unknownError));
      setStatus('error');
    }
  };

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
            <dt>Install version</dt>
            <dd>{report?.install?.version ?? 'Not scanned yet'}</dd>
          </div>
          <div>
            <dt>Documents root</dt>
            <dd>{report?.documents?.root ?? 'Not scanned yet'}</dd>
          </div>
          <div>
            <dt>Errors</dt>
            <dd>{report?.errors?.length ?? 0}</dd>
          </div>
        </dl>
      </section>

      <section aria-labelledby="scan-heading" className="panel">
        <div className="panel-header">
          <h2 id="scan-heading">Scan</h2>
          <button type="button" onClick={handleScan} disabled={status === 'loading'}>
            {status === 'loading' ? 'Scanning…' : 'Scan Local Files'}
          </button>
        </div>

        {error ? (
          <p role="alert" className="error">
            {error}
          </p>
        ) : null}
      </section>

      <section aria-labelledby="raw-heading" className="panel">
        <h2 id="raw-heading">Raw JSON</h2>
        <pre className="json-view">{report ? JSON.stringify(report, null, 2) : 'No scan yet.'}</pre>
      </section>
    </>
  );
}
