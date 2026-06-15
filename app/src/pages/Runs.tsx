import { useEffect, useState } from 'react';

import {
  clearFactOverride,
  clearRunNote,
  loadRunFacts,
  loadRunNotes,
  loadRuns,
  rescanSaves,
  setFactOverride,
  setRunNote,
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
          <h2 id="runs-heading">Campaign Archives</h2>
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

          <RunFactPanel
            run={selectedRun}
            facts={facts}
            factStatus={factStatus}
            onFactsChanged={() => {
              if (selectedRunPath) {
                setFactStatus('loading');
                loadRunFacts(selectedRunPath)
                  .then((loadedFacts) => {
                    setFacts(loadedFacts);
                    setFactStatus('ready');
                  })
                  .catch((err) => {
                    setError(errorMessage(err));
                    setFactStatus('error');
                  });
              }
            }}
          />
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
  onFactsChanged,
}: {
  run: PersistedRunSummary | null;
  facts: RunFactSummary[];
  factStatus: LoadState;
  onFactsChanged: () => void;
}) {
  const [showAllFacts, setShowAllFacts] = useState(false);
  const [editingFact, setEditingFact] = useState<{ dimension: string; key: string } | null>(null);
  const [editValue, setEditValue] = useState('');
  const [editReason, setEditReason] = useState('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [newDimension, setNewDimension] = useState('');
  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [newReason, setNewReason] = useState('');
  const [overrideError, setOverrideError] = useState<string | null>(null);
  const [runNote, setRunNoteText] = useState('');
  const [runNoteSaved, setRunNoteSaved] = useState(false);

  useEffect(() => {
    if (!run) return;
    setRunNoteSaved(false);
    loadRunNotes(run.folder_path)
      .then((note) => {
        setRunNoteText(note?.note_text ?? '');
      })
      .catch(() => {
        // Non-critical — ignore
      });
  }, [run?.folder_path]);

  if (!run) {
    return <aside className="run-fact-panel muted">Select a run to view parsed facts.</aside>;
  }

  const visibleFacts = showAllFacts ? facts : facts.slice(0, 12);

  const handleEditClick = (fact: RunFactSummary) => {
    setEditingFact({ dimension: fact.dimension, key: fact.key });
    setEditValue(JSON.stringify(fact.value));
    setEditReason('');
    setOverrideError(null);
  };

  const handleSaveEdit = async () => {
    if (!editingFact || !run) return;
    setOverrideError(null);
    try {
      await setFactOverride(
        run.folder_path,
        editingFact.dimension,
        editingFact.key,
        editValue,
        editReason || null,
      );
      setEditingFact(null);
      onFactsChanged();
    } catch (err) {
      setOverrideError(errorMessage(err));
    }
  };

  const handleCancelEdit = () => {
    setEditingFact(null);
    setOverrideError(null);
  };

  const handleClearOverride = async (fact: RunFactSummary) => {
    if (!run) return;
    setOverrideError(null);
    try {
      await clearFactOverride(run.folder_path, fact.dimension, fact.key);
      onFactsChanged();
    } catch (err) {
      setOverrideError(errorMessage(err));
    }
  };

  const handleAddFact = async () => {
    if (!run || !newDimension || !newKey || !newValue) return;
    setOverrideError(null);
    try {
      await setFactOverride(
        run.folder_path,
        newDimension,
        newKey,
        newValue,
        newReason || null,
      );
      setShowAddForm(false);
      setNewDimension('');
      setNewKey('');
      setNewValue('');
      setNewReason('');
      onFactsChanged();
    } catch (err) {
      setOverrideError(errorMessage(err));
    }
  };

  return (
    <aside className="run-fact-panel" aria-label="Selected run facts">
      <h3>{run.display_name ?? run.run_folder}</h3>
      <dl className="run-detail-list">
        <dt>Folder</dt>
        <dd>{run.folder_path}</dd>
        <dt>Latest save</dt>
        <dd>{run.latest_save_path ?? 'Unknown'}</dd>
      </dl>

      <div className="run-note-section">
        <h4>Run notes</h4>
        <textarea
          className="filter-input"
          rows={4}
          style={{ resize: 'vertical' }}
          value={runNote}
          onChange={(e) => {
            setRunNoteText(e.target.value);
            setRunNoteSaved(false);
          }}
          placeholder="Add notes for this run…"
        />
        <div className="run-note-actions">
          <button
            type="button"
            onClick={async () => {
              if (!run) return;
              setOverrideError(null);
              try {
                await setRunNote(run.folder_path, runNote);
                setRunNoteSaved(true);
              } catch (err) {
                setOverrideError(errorMessage(err));
              }
            }}
          >
            {runNoteSaved ? 'Saved' : 'Save note'}
          </button>
          <button
            type="button"
            className="secondary-button"
            onClick={async () => {
              if (!run) return;
              setOverrideError(null);
              try {
                await clearRunNote(run.folder_path);
                setRunNoteText('');
                setRunNoteSaved(false);
              } catch (err) {
                setOverrideError(errorMessage(err));
              }
            }}
          >
            Clear
          </button>
        </div>
      </div>

      <h4>Parsed facts</h4>
      {factStatus === 'loading' ? <p className="muted">Loading facts…</p> : null}
      {overrideError ? <p role="alert" className="error">{overrideError}</p> : null}
      {facts.length > 0 ? (
        <ul className="fact-list">
          {visibleFacts.map((fact) => {
            const isEditing =
              editingFact?.dimension === fact.dimension && editingFact?.key === fact.key;
            return (
              <li key={`${fact.dimension}:${fact.key}`}>
                {isEditing ? (
                  <div className="fact-edit-form">
                    <span className="fact-key">{fact.dimension}.{fact.key}</span>
                    <input
                      type="text"
                      value={editValue}
                      onChange={(e) => setEditValue(e.target.value)}
                      placeholder="Value (JSON)"
                      className="filter-input"
                    />
                    <input
                      type="text"
                      value={editReason}
                      onChange={(e) => setEditReason(e.target.value)}
                      placeholder="Reason (optional)"
                      className="filter-input"
                    />
                    <div className="fact-edit-actions">
                      <button type="button" onClick={handleSaveEdit}>Save</button>
                      <button type="button" className="secondary-button" onClick={handleCancelEdit}>
                        Cancel
                      </button>
                    </div>
                  </div>
                ) : (
                  <>
                    <span className="fact-key">
                      {fact.dimension}.{fact.key}
                      {fact.is_override ? <span className="badge badge-override">override</span> : null}
                    </span>
                    <span className="fact-value">{formatFactValue(fact.value)}</span>
                    <span className="muted">{fact.confidence}</span>
                    <div className="fact-actions">
                      <button
                        type="button"
                        className="link-button"
                        onClick={() => handleEditClick(fact)}
                      >
                        Edit
                      </button>
                      {fact.is_override ? (
                        <button
                          type="button"
                          className="link-button"
                          onClick={() => handleClearOverride(fact)}
                        >
                          Clear override
                        </button>
                      ) : null}
                    </div>
                  </>
                )}
              </li>
            );
          })}
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

      <h4>Fact overrides</h4>
      {showAddForm ? (
        <div className="fact-add-form">
          <input
            type="text"
            value={newDimension}
            onChange={(e) => setNewDimension(e.target.value)}
            placeholder="Dimension (e.g. empire)"
            className="filter-input"
          />
          <input
            type="text"
            value={newKey}
            onChange={(e) => setNewKey(e.target.value)}
            placeholder="Key (e.g. origin)"
            className="filter-input"
          />
          <input
            type="text"
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
            placeholder="Value (JSON, e.g. &quot;origin_synaptic&quot;)"
            className="filter-input"
          />
          <input
            type="text"
            value={newReason}
            onChange={(e) => setNewReason(e.target.value)}
            placeholder="Reason (optional)"
            className="filter-input"
          />
          <div className="fact-edit-actions">
            <button type="button" onClick={handleAddFact}>Add override</button>
            <button
              type="button"
              className="secondary-button"
              onClick={() => setShowAddForm(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : (
        <button type="button" className="secondary-button" onClick={() => setShowAddForm(true)}>
          Add fact override
        </button>
      )}
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
