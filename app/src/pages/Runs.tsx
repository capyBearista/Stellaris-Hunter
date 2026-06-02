import { useState } from 'react';

import { scanLocalState, type ScanReport } from '../tauri';

type LoadState = 'idle' | 'loading' | 'ready' | 'error';

export function Runs() {
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

  const saveRuns = report?.documents?.save_runs ?? [];

  return (
    <section aria-labelledby="runs-heading" className="panel">
      <div className="panel-header">
        <h2 id="runs-heading">Runs / Saves</h2>
        <button type="button" onClick={handleScan} disabled={status === 'loading'}>
          {status === 'loading' ? 'Scanning…' : 'Scan Local Files'}
        </button>
      </div>

      {error ? <p role="alert" className="error">{error}</p> : null}

      {saveRuns.length > 0 ? (
        <ul className="run-list">
          {saveRuns.map((run) => (
            <li key={run.run_folder} className="run-list-item">
              <div className="run-list-main">
                <strong>{run.run_folder}</strong>
                <span>{run.latest_save?.name ?? 'No latest save'}</span>
              </div>
              <div className="run-list-meta">
                {run.latest_save?.ironman === true ? (
                  <span className="badge badge-eligible" title="Ironman mode enabled">Ironman</span>
                ) : null}
                {run.latest_save?.cheated_on_save === true ? (
                  <span className="badge badge-ineligible" title="Cheats were used">Cheated</span>
                ) : null}
                {run.eligibility ? (
                  <EligibilityBadge eligibility={run.eligibility} />
                ) : null}
              </div>
              {run.issues && run.issues.length > 0 ? (
                <ul className="run-issues">
                  {run.issues.map((issue, i) => (
                    <li key={i} className="muted">
                      {issue}
                    </li>
                  ))}
                </ul>
              ) : null}
            </li>
          ))}
        </ul>
      ) : (
        <p className="muted">No runs loaded.</p>
      )}
    </section>
  );
}

// ---------------------------------------------------------------------------
// Eligibility badge sub-component
// ---------------------------------------------------------------------------

import type { SaveEligibility } from '../tauri';

function EligibilityBadge({ eligibility }: { eligibility: SaveEligibility }) {
  const badgeClass = (() => {
    switch (eligibility.conclusion) {
      case 'likely_eligible':
        return 'badge badge-eligible';
      case 'likely_ineligible':
        return 'badge badge-ineligible';
      default:
        return 'badge badge-unknown';
    }
  })();

  const modRiskText = (() => {
    switch (eligibility.mod_risk) {
      case 'checksum_scoped':
        return ' ⚠ Mod conflict risk';
      case 'unknown':
        return ' ? Unknown mod risk';
      default:
        return '';
    }
  })();

  return (
    <span className={badgeClass} title={eligibility.reasons.join('; ')}>
      {eligibility.conclusion === 'likely_eligible'
        ? 'Eligible'
        : eligibility.conclusion === 'likely_ineligible'
          ? 'Ineligible'
          : 'Unknown'}
      {modRiskText}
    </span>
  );
}
