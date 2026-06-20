import { useEffect, useState } from 'react';
import { invalidateScanCache } from '../scanCache';
import {
  loadAppConfig,
  saveAppConfig,
  loadAppInfo,
  syncCatalog,
  syncSteamAchievements,
  syncIcons,
  rescanSaves,
} from '../tauri';
import type { AppConfig, AppInfo, CatalogSyncResult, SteamSyncResult, IconSyncResult } from '../tauri';
import type { PersistedRunSummary } from '../tauri';

type SyncState = 'idle' | 'loading' | 'success' | 'error';

export function Settings() {
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [config, setConfig] = useState<AppConfig>({ installPathOverride: null, documentsPathOverride: null });
  const [installPath, setInstallPath] = useState('');
  const [documentsPath, setDocumentsPath] = useState('');
  const [configSaving, setConfigSaving] = useState(false);
  const [configMessage, setConfigMessage] = useState<{ kind: 'success' | 'error'; text: string } | null>(null);

  // Sync states
  const [catalogSync, setCatalogSync] = useState<SyncState>('idle');
  const [catalogMsg, setCatalogMsg] = useState<string | null>(null);
  const [steamSync, setSteamSync] = useState<SyncState>('idle');
  const [steamMsg, setSteamMsg] = useState<string | null>(null);
  const [iconSync, setIconSync] = useState<SyncState>('idle');
  const [iconMsg, setIconMsg] = useState<string | null>(null);
  const [saveRescan, setSaveRescan] = useState<SyncState>('idle');
  const [saveMsg, setSaveMsg] = useState<string | null>(null);

  useEffect(() => {
    loadAppInfo()
      .then(setAppInfo)
      .catch((err) => console.error('loadAppInfo failed:', err));
    loadAppConfig()
      .then((cfg) => {
        setConfig(cfg);
        setInstallPath(cfg.installPathOverride ?? '');
        setDocumentsPath(cfg.documentsPathOverride ?? '');
      })
      .catch((err) => console.error('loadAppConfig failed:', err));
  }, []);

  const handleSaveConfig = async () => {
    setConfigSaving(true);
    setConfigMessage(null);
    try {
      const newConfig: AppConfig = {
        installPathOverride: installPath.trim() || null,
        documentsPathOverride: documentsPath.trim() || null,
      };
      await saveAppConfig(newConfig);
      setConfig(newConfig);
      invalidateScanCache();
      setConfigMessage({ kind: 'success', text: 'Configuration saved.' });
    } catch (err) {
      setConfigMessage({ kind: 'error', text: `Save failed: ${err}` });
    } finally {
      setConfigSaving(false);
    }
  };

  const handleSyncCatalog = async () => {
    setCatalogSync('loading');
    setCatalogMsg(null);
    try {
      const result: CatalogSyncResult = await syncCatalog();
      setCatalogSync('success');
      setCatalogMsg(result.message);
    } catch (err) {
      setCatalogSync('error');
      setCatalogMsg(`Sync failed: ${err}`);
    }
  };

  const handleSyncSteam = async () => {
    setSteamSync('loading');
    setSteamMsg(null);
    try {
      const result: SteamSyncResult = await syncSteamAchievements();
      setSteamSync('success');
      setSteamMsg(result.message);
    } catch (err) {
      setSteamSync('error');
      setSteamMsg(`Sync failed: ${err}`);
    }
  };

  const handleSyncIcons = async () => {
    setIconSync('loading');
    setIconMsg(null);
    try {
      const result: IconSyncResult = await syncIcons();
      setIconSync('success');
      setIconMsg(result.message);
    } catch (err) {
      setIconSync('error');
      setIconMsg(`Sync failed: ${err}`);
    }
  };

  const handleRescanSaves = async () => {
    setSaveRescan('loading');
    setSaveMsg(null);
    try {
      const runs: PersistedRunSummary[] = await rescanSaves();
      setSaveRescan('success');
      invalidateScanCache();
      setSaveMsg(`Scan complete — ${runs.length} run(s) found.`);
    } catch (err) {
      setSaveRescan('error');
      setSaveMsg(`Rescan failed: ${err}`);
    }
  };

  const syncStatusClass = (state: SyncState) => {
    if (state === 'loading') return 'status status-loading';
    if (state === 'error') return 'status status-error';
    if (state === 'success') return 'status status-ready';
    return 'status';
  };

  const syncStatusLabel = (state: SyncState) => {
    if (state === 'loading') return 'Running…';
    if (state === 'error') return 'Failed';
    if (state === 'success') return 'Done';
    return '';
  };

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Configuration</h2>
      </div>

      {/* App Info */}
      <div className="run-fact-panel" style={{ marginBottom: '1rem' }}>
        <h3>App Info</h3>
        <dl className="run-detail-list">
          <dt>App Version</dt>
          <dd>{appInfo?.appVersion ?? '…'}</dd>
          <dt>Catalog Version</dt>
          <dd>{appInfo?.catalogVersion ?? '—'}</dd>
          <dt>Stellaris Version</dt>
          <dd>{appInfo?.stellarisVersion ?? '—'}</dd>
          <dt>Last Catalog Sync</dt>
          <dd>{appInfo?.lastCatalogSync ?? '—'}</dd>
          <dt>Last Steam Sync</dt>
          <dd>{appInfo?.lastSteamSync ?? '—'}</dd>
          <dt>Last Save Scan</dt>
          <dd>{appInfo?.lastSaveScan ?? '—'}</dd>
        </dl>
      </div>

      {/* Sync Actions */}
      <div className="run-fact-panel" style={{ marginBottom: '1rem' }}>
        <h3>Sync Actions</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
            <button onClick={handleSyncCatalog} disabled={catalogSync === 'loading'}>
              {catalogSync === 'loading' ? 'Syncing…' : 'Sync Catalog'}
            </button>
            <span className={syncStatusClass(catalogSync)}>{syncStatusLabel(catalogSync)}</span>
            {catalogMsg && <span className="muted">{catalogMsg}</span>}
          </div>
          <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
            <button onClick={handleSyncSteam} disabled={steamSync === 'loading'}>
              {steamSync === 'loading' ? 'Syncing…' : 'Sync Steam'}
            </button>
            <span className={syncStatusClass(steamSync)}>{syncStatusLabel(steamSync)}</span>
            {steamMsg && <span className="muted">{steamMsg}</span>}
          </div>
          <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
            <button onClick={handleSyncIcons} disabled={iconSync === 'loading'}>
              {iconSync === 'loading' ? 'Syncing…' : 'Sync Icons'}
            </button>
            <span className={syncStatusClass(iconSync)}>{syncStatusLabel(iconSync)}</span>
            {iconMsg && <span className="muted">{iconMsg}</span>}
          </div>
          <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center' }}>
            <button onClick={handleRescanSaves} disabled={saveRescan === 'loading'}>
              {saveRescan === 'loading' ? 'Scanning…' : 'Rescan Saves'}
            </button>
            <span className={syncStatusClass(saveRescan)}>{syncStatusLabel(saveRescan)}</span>
            {saveMsg && <span className="muted">{saveMsg}</span>}
          </div>
        </div>
      </div>

      {/* Path Overrides */}
      <div className="run-fact-panel" style={{ marginBottom: '1rem' }}>
        <h3>Path Overrides</h3>
        <div className="fact-edit-form">
          <label>
            Install Path Override
            <input
              className="filter-input"
              type="text"
              placeholder="e.g. /path/to/Steam/steamapps/common/Stellaris"
              value={installPath}
              onChange={(e) => setInstallPath(e.target.value)}
            />
          </label>
          <label>
            Documents Path Override
            <input
              className="filter-input"
              type="text"
              placeholder="e.g. /path/to/Documents/Paradox Interactive/Stellaris"
              value={documentsPath}
              onChange={(e) => setDocumentsPath(e.target.value)}
            />
          </label>
          <div className="fact-edit-actions">
            <button onClick={handleSaveConfig} disabled={configSaving}>
              {configSaving ? 'Saving…' : 'Save'}
            </button>
            {configMessage && (
              <span className={configMessage.kind === 'error' ? 'error' : 'status status-ready'}>
                {configMessage.text}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Attribution */}
      <div className="run-fact-panel">
        <h3>Attribution</h3>
        <p className="muted">
          Stellaris is a game by{' '}
          <a href="https://www.paradoxinteractive.com/games/stellaris/about" target="_blank" rel="noopener noreferrer" className="link-button">
            Paradox Interactive
          </a>
          .
        </p>
        <p className="muted">
          Achievement data synced via{' '}
          <a href="https://store.steampowered.com" target="_blank" rel="noopener noreferrer" className="link-button">
            Steam
          </a>
          .
        </p>
        <p className="muted">
          This project is licensed under{' '}
          <a href="https://www.mozilla.org/en-US/MPL/2.0/" target="_blank" rel="noopener noreferrer" className="link-button">
            MPL-2.0
          </a>
          .
        </p>
        <p className="muted">Built with Tauri, React, and Rust.</p>
      </div>
    </section>
  );
}
