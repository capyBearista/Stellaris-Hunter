import { useEffect, useMemo, useState } from 'react';

import {
  loadAchievements,
  loadCatalogInfo,
  loadCompletionOverrides,
  setCompletionOverride,
  clearCompletionOverride,
  syncCatalog,
  syncSteamAchievements,
  getAchievementIcon,
  syncIcons,
  type AchievementEntry,
  type AchievementOverride,
  type CatalogInfo,
  type CatalogSyncResult,
  type IconSyncResult,
  type SteamSyncResult,
} from '../tauri';
import { IconPlaceholder } from '../components/IconPlaceholder';

const DIFFICULTIES = ['All', 'VE', 'E', 'M', 'H', 'VH', 'I', 'UC'] as const;
type SortKey = 'name' | 'group' | 'difficulty' | 'version';
type SortDir = 'asc' | 'desc';

export function Achievements() {
  const [achievements, setAchievements] = useState<AchievementEntry[]>([]);
  const [catalogInfo, setCatalogInfo] = useState<CatalogInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [overrides, setOverrides] = useState<Record<string, boolean>>({});
  const [overrideError, setOverrideError] = useState<string | null>(null);

  // Sync catalog
  const [syncing, setSyncing] = useState(false);
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);

  // Steam sync
  const [steamSyncing, setSteamSyncing] = useState(false);
  const [steamSyncMessage, setSteamSyncMessage] = useState<string | null>(null);
  const [steamSyncError, setSteamSyncError] = useState<string | null>(null);

  // Icon sync
  const [iconSyncing, setIconSyncing] = useState(false);
  const [iconSyncMessage, setIconSyncMessage] = useState<string | null>(null);
  const [iconSyncError, setIconSyncError] = useState<string | null>(null);
  const [iconVersion, setIconVersion] = useState(0);

  const handleSyncSteam = async () => {
    setSteamSyncing(true);
    setSteamSyncMessage(null);
    setSteamSyncError(null);
    try {
      const result: SteamSyncResult = await syncSteamAchievements();
      setSteamSyncMessage(result.message);
      // Reload achievements after successful sync
      const [ach, cat, ovr] = await Promise.all([
        loadAchievements(),
        loadCatalogInfo(),
        loadCompletionOverrides(),
      ]);
      setAchievements(ach);
      setCatalogInfo(cat);
      setOverrides(toOverrideRecord(ovr));
    } catch (unknownError) {
      setSteamSyncError(
        unknownError instanceof Error ? unknownError.message : String(unknownError),
      );
    } finally {
      setSteamSyncing(false);
    }
  };

  const handleSyncIcons = async () => {
    setIconSyncing(true);
    setIconSyncMessage(null);
    setIconSyncError(null);
    try {
      const result: IconSyncResult = await syncIcons();
      setIconSyncMessage(result.message);
      // Force re-render of icon hooks by toggling a counter
      setIconVersion((v) => v + 1);
    } catch (unknownError) {
      setIconSyncError(
        unknownError instanceof Error ? unknownError.message : String(unknownError),
      );
    } finally {
      setIconSyncing(false);
    }
  };

  const handleSyncCatalog = async () => {
    setSyncing(true);
    setSyncMessage(null);
    setSyncError(null);
    try {
      const result: CatalogSyncResult = await syncCatalog();
      setSyncMessage(result.message);
      if (result.updated) {
        // Reload achievements after successful sync
        const [ach, cat, ovr] = await Promise.all([
          loadAchievements(),
          loadCatalogInfo(),
          loadCompletionOverrides(),
        ]);
        setAchievements(ach);
        setCatalogInfo(cat);
        setOverrides(toOverrideRecord(ovr));
      }
    } catch (unknownError) {
      setSyncError(
        unknownError instanceof Error ? unknownError.message : String(unknownError),
      );
    } finally {
      setSyncing(false);
    }
  };

  // Filters
  const [search, setSearch] = useState('');
  const [groupFilter, setGroupFilter] = useState('All');
  const [difficultyFilter, setDifficultyFilter] = useState('All');

  // Sort
  const [sortKey, setSortKey] = useState<SortKey>('name');
  const [sortDir, setSortDir] = useState<SortDir>('asc');

  // Expanded rows
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const [ach, cat, ovr] = await Promise.all([
          loadAchievements(),
          loadCatalogInfo(),
          loadCompletionOverrides(),
        ]);
        if (!cancelled) {
          setAchievements(ach);
          setCatalogInfo(cat);
          setOverrides(toOverrideRecord(ovr));
        }
      } catch (unknownError) {
        if (!cancelled) {
          setError(unknownError instanceof Error ? unknownError.message : String(unknownError));
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    load();
    return () => {
      cancelled = true;
    };
  }, []);

  const groups = useMemo(() => {
    const g = new Set<string>();
    g.add('All');
    for (const a of achievements) {
      if (a.source.group) g.add(a.source.group);
    }
    return [...g].sort();
  }, [achievements]);

  const toggleExpanded = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleCompletionToggle = async (id: string, currentlyCompleted: boolean) => {
    setOverrideError(null);
    try {
      if (currentlyCompleted) {
        await clearCompletionOverride(id);
        setOverrides((prev) => {
          const next = { ...prev };
          delete next[id];
          return next;
        });
      } else {
        await setCompletionOverride(id, true);
        setOverrides((prev) => ({ ...prev, [id]: true }));
      }
    } catch (unknownError) {
      setOverrideError(
        unknownError instanceof Error ? unknownError.message : String(unknownError),
      );
    }
  };

  const handleSort = (key: SortKey) => {
    setSortDir((prev) => (sortKey === key ? (prev === 'asc' ? 'desc' : 'asc') : 'asc'));
    setSortKey(key);
  };

  const sortArrow = (key: SortKey): string => {
    if (sortKey !== key) return '';
    return sortDir === 'asc' ? ' ▲' : ' ▼';
  };

  const filtered = useMemo(() => {
    const q = search.toLowerCase();
    return achievements
      .filter((a) => {
        if (q && !a.source.name.toLowerCase().includes(q)) return false;
        if (groupFilter !== 'All' && a.source.group !== groupFilter) return false;
        if (difficultyFilter !== 'All' && (a.source.difficulty ?? 'UC') !== difficultyFilter)
          return false;
        return true;
      })
      .sort((a, b) => {
        let cmp = 0;
        switch (sortKey) {
          case 'name':
            cmp = a.source.name.localeCompare(b.source.name);
            break;
          case 'group':
            cmp = (a.source.group ?? '').localeCompare(b.source.group ?? '');
            break;
          case 'difficulty':
            cmp = (a.source.difficulty ?? 'UC').localeCompare(b.source.difficulty ?? 'UC');
            break;
          case 'version':
            cmp = (a.source.version_added ?? '').localeCompare(b.source.version_added ?? '');
            break;
        }
        return sortDir === 'asc' ? cmp : -cmp;
      });
  }, [achievements, search, groupFilter, difficultyFilter, sortKey, sortDir]);

  const diffClass = (d: string | null): string => {
    const diff = d ?? 'UC';
    if (diff === 'VE' || diff === 'E') return 'badge badge-easy';
    if (diff === 'M') return 'badge badge-medium';
    if (diff === 'H' || diff === 'VH') return 'badge badge-hard';
    return 'badge badge-insane';
  };

  if (loading) {
    return (
      <section className="panel">
        <p className="muted">Loading achievements…</p>
      </section>
    );
  }

  if (error) {
    return (
      <section className="panel">
        <h2>Achievements</h2>
        <p role="alert" className="error">
          {error}
        </p>
      </section>
    );
  }

  return (
    <section className="panel">
      <div className="panel-header">
        <h2>Achievements</h2>
        <span className="muted">{achievements.length} total &middot; {filtered.length} shown</span>
      </div>

      {catalogInfo ? (
        <div className="catalog-info-row">
          <p className="catalog-info muted">
            Catalog v{catalogInfo.catalog_version}
            {catalogInfo.stellaris_version ? ` · Stellaris ${catalogInfo.stellaris_version}` : ''}
            {catalogInfo.updated_at ? ` · Updated ${catalogInfo.updated_at}` : ''}
          </p>
          <div className="sync-btn-group">
            <button className="sync-btn" onClick={handleSyncCatalog} disabled={syncing}>
              {syncing ? 'Syncing…' : 'Sync Catalog'}
            </button>
            <button className="sync-btn" onClick={handleSyncSteam} disabled={steamSyncing}>
              {steamSyncing ? 'Syncing…' : 'Sync Steam'}
            </button>
            <button className="sync-btn" onClick={handleSyncIcons} disabled={iconSyncing}>
              {iconSyncing ? 'Syncing…' : 'Sync Icons'}
            </button>
          </div>
        </div>
      ) : null}
      {syncMessage ? <p className="sync-message">{syncMessage}</p> : null}
      {syncError ? (
        <p role="alert" className="error" style={{ margin: '0.5rem 0' }}>
          Sync error: {syncError}
        </p>
      ) : null}
      {steamSyncMessage ? <p className="sync-message">{steamSyncMessage}</p> : null}
      {steamSyncError ? (
        <p role="alert" className="error" style={{ margin: '0.5rem 0' }}>
          Steam sync error: {steamSyncError}
        </p>
      ) : null}
      {iconSyncMessage ? <p className="sync-message">{iconSyncMessage}</p> : null}
      {iconSyncError ? (
        <p role="alert" className="error" style={{ margin: '0.5rem 0' }}>
          Icon sync error: {iconSyncError}
        </p>
      ) : null}

      <div className="achievement-filters">
        <input
          type="search"
          placeholder="Search by name…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="filter-input"
        />
        <select
          value={groupFilter}
          onChange={(e) => setGroupFilter(e.target.value)}
          className="filter-select"
        >
          {groups.map((g) => (
            <option key={g} value={g}>
              {g}
            </option>
          ))}
        </select>
        <select
          value={difficultyFilter}
          onChange={(e) => setDifficultyFilter(e.target.value)}
          className="filter-select"
        >
          {DIFFICULTIES.map((d) => (
            <option key={d} value={d}>
              {d}
            </option>
          ))}
        </select>
      </div>

      {overrideError ? (
        <p role="alert" className="error" style={{ marginBottom: '0.75rem' }}>
          Override error: {overrideError}
        </p>
      ) : null}

      {filtered.length === 0 ? (
        <p className="muted">No achievements match the current filters.</p>
      ) : (
        <table className="achievement-table">
          <thead>
            <tr>
              <th style={{ width: 40 }}>Icon</th>
              <th onClick={() => handleSort('name')} className="sortable">
                Name{sortArrow('name')}
              </th>
              <th onClick={() => handleSort('group')} className="sortable">
                DLC Group{sortArrow('group')}
              </th>
              <th onClick={() => handleSort('difficulty')} className="sortable">
                Difficulty{sortArrow('difficulty')}
              </th>
              <th onClick={() => handleSort('version')} className="sortable">
                Version Added{sortArrow('version')}
              </th>
              <th>Completion</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((ach) => (
              <AchievementRow
                key={ach.id}
                ach={ach}
                expanded={expanded.has(ach.id)}
                onToggle={() => toggleExpanded(ach.id)}
                diffClass={diffClass}
                completed={overrides[ach.id] ?? ach.completed ?? false}
                onCompletionToggle={() => handleCompletionToggle(ach.id, overrides[ach.id] ?? false)}
                iconVersion={iconVersion}
              />
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

// ---------------------------------------------------------------------------
// Row component (extracted for clarity)
// ---------------------------------------------------------------------------

interface AchievementRowProps {
  ach: AchievementEntry;
  expanded: boolean;
  onToggle: () => void;
  diffClass: (d: string | null) => string;
  completed: boolean;
  onCompletionToggle: () => void;
  iconVersion: number;
}

function toOverrideRecord(overrides: AchievementOverride[]): Record<string, boolean> {
  const record: Record<string, boolean> = {};
  for (const o of overrides) {
    // Checkbox MVP only exposes force-completed vs no manual override.
    // A future three-state UI should preserve completed=false as force-incomplete.
    if (o.completed) record[o.achievement_id] = true;
  }
  return record;
}

// ---------------------------------------------------------------------------
// Icon component (fetches and caches blob URL per achievement)
// ---------------------------------------------------------------------------

function AchievementIcon({
  achievementId,
  iconVersion,
}: {
  achievementId: string;
  iconVersion: number;
}) {
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    getAchievementIcon(achievementId).then((bytes) => {
      if (cancelled || !bytes) return;
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/png' });
      const blobUrl = URL.createObjectURL(blob);
      if (!cancelled) {
        setUrl(blobUrl);
      } else {
        URL.revokeObjectURL(blobUrl);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [achievementId, iconVersion]);

  return url ? (
    <img src={url} alt="" width={32} height={32} style={{ borderRadius: 4 }} />
  ) : (
    <IconPlaceholder />
  );
}

function AchievementRow({
  ach,
  expanded,
  onToggle,
  diffClass,
  completed,
  onCompletionToggle,
  iconVersion,
}: AchievementRowProps) {
  return (
    <>
      <tr
        className="achievement-row"
        onClick={onToggle}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onToggle();
          }
        }}
      >
        <td style={{ width: 40 }}>
          <AchievementIcon achievementId={ach.id} iconVersion={iconVersion} />
        </td>
        <td>{ach.source.name}</td>
        <td>{ach.source.group ?? '—'}</td>
        <td>
          <span className={diffClass(ach.source.difficulty)}>
            {ach.source.difficulty ?? 'UC'}
          </span>
        </td>
        <td>{ach.source.version_added ?? '—'}</td>
        <td>
          <input
            type="checkbox"
            checked={completed}
            onChange={onCompletionToggle}
            onClick={(e) => e.stopPropagation()}
            onKeyDown={(e) => e.stopPropagation()}
            title={completed ? 'Clear manual completion' : 'Mark as completed'}
          />
        </td>
      </tr>
      {expanded ? (
        <tr className="achievement-detail-row">
          <td colSpan={6}>
            <div className="achievement-detail">
              {ach.source.description ? (
                <p>
                  <strong>Description:</strong> {ach.source.description}
                </p>
              ) : null}
              {ach.source.requirement ? (
                <p>
                  <strong>Requirement:</strong> {ach.source.requirement}
                </p>
              ) : null}
              {ach.source.hint ? (
                <p>
                  <strong>Hint:</strong> {ach.source.hint}
                </p>
              ) : null}
              {ach.curation.tags.length > 0 ? (
                <p>
                  <strong>Tags:</strong> {ach.curation.tags.join(', ')}
                </p>
              ) : null}
              {ach.curation.warnings.length > 0 ? (
                <p className="detail-warning">
                  <strong>Warnings:</strong> {ach.curation.warnings.join('; ')}
                </p>
              ) : null}
            </div>
          </td>
        </tr>
      ) : null}
    </>
  );
}
