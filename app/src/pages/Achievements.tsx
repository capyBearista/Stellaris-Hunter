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
  type LauncherDlcSummary,
  type ScanReport,
  type SteamSyncResult,
} from '../tauri';
import { IconPlaceholder } from '../components/IconPlaceholder';
import { getCachedScanReport, scanLocalStateCached } from '../scanCache';

// Icon cache keyed by achievementId + iconVersion to avoid redundant IPC calls.
const iconCache = new Map<string, string | null>();
const inflightIconFetches = new Map<string, Promise<string | null>>();

const DIFFICULTIES = ['All', 'VE', 'E', 'M', 'H', 'VH', 'I', 'UC'] as const;
const DIFFICULTY_LABELS: Record<string, string> = {
  VE: 'Very Easy',
  E: 'Easy',
  M: 'Medium',
  H: 'Hard',
  VH: 'Very Hard',
  I: 'Insane',
  UC: 'Unclassified',
};

const COLUMN_CONFIG = [
  { key: 'completion', label: 'Completion', defaultVisible: true },
  { key: 'icon', label: 'Icon', defaultVisible: true },
  { key: 'group', label: 'Group', defaultVisible: true },
  { key: 'difficulty', label: 'Difficulty', defaultVisible: true },
  { key: 'tags', label: 'Tags', defaultVisible: false },
  { key: 'ruleConfidence', label: 'Rule Confidence', defaultVisible: false },
  { key: 'warnings', label: 'Warnings', defaultVisible: true },
  { key: 'version', label: 'Version Added', defaultVisible: false },
  { key: 'steamApi', label: 'Steam API Name', defaultVisible: false },
] as const;

type ColumnKey = (typeof COLUMN_CONFIG)[number]['key'];
type ViewMode = 'list' | 'board';
type CompletionFilter = 'all' | 'completed' | 'incomplete';
type DlcAvailabilityFilter = 'all' | 'attention' | 'available' | 'unknown';
type SortKey = 'name' | 'group' | 'difficulty';
type SortDir = 'asc' | 'desc';

export function Achievements() {
  const [achievements, setAchievements] = useState<AchievementEntry[]>([]);
  const [catalogInfo, setCatalogInfo] = useState<CatalogInfo | null>(null);
  const [scanReport, setScanReport] = useState<ScanReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [overrides, setOverrides] = useState<Record<string, boolean>>({});
  const [overrideError, setOverrideError] = useState<string | null>(null);

  const [syncing, setSyncing] = useState(false);
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);

  const [steamSyncing, setSteamSyncing] = useState(false);
  const [steamSyncMessage, setSteamSyncMessage] = useState<string | null>(null);
  const [steamSyncError, setSteamSyncError] = useState<string | null>(null);

  const [iconSyncing, setIconSyncing] = useState(false);
  const [iconSyncMessage, setIconSyncMessage] = useState<string | null>(null);
  const [iconSyncError, setIconSyncError] = useState<string | null>(null);
  const [iconVersion, setIconVersion] = useState(0);

  const [search, setSearch] = useState('');
  const [completionFilter, setCompletionFilter] = useState<CompletionFilter>('all');
  const [groupFilter, setGroupFilter] = useState('All');
  const [dlcAvailabilityFilter, setDlcAvailabilityFilter] = useState<DlcAvailabilityFilter>('all');
  const [difficultyFilter, setDifficultyFilter] = useState('All');
  const [sortKey, setSortKey] = useState<SortKey>('name');
  const [sortDir, setSortDir] = useState<SortDir>('asc');
  const [viewMode, setViewMode] = useState<ViewMode>('list');
  const [showColumns, setShowColumns] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const DEFAULT_COLUMNS = useMemo(
    () =>
      Object.fromEntries(COLUMN_CONFIG.map((column) => [column.key, column.defaultVisible])) as Record<
        ColumnKey,
        boolean
      >,
    [],
  );
  const [visibleColumns, setVisibleColumns] = useState<Record<ColumnKey, boolean>>(() => ({ ...DEFAULT_COLUMNS }));

  const columnsChanged = useMemo(
    () => COLUMN_CONFIG.some((column) => visibleColumns[column.key] !== DEFAULT_COLUMNS[column.key]),
    [visibleColumns, DEFAULT_COLUMNS],
  );

  const resetColumns = () => setVisibleColumns({ ...DEFAULT_COLUMNS });

  const handleSyncSteam = async () => {
    setSteamSyncing(true);
    setSteamSyncMessage(null);
    setSteamSyncError(null);
    try {
      const result: SteamSyncResult = await syncSteamAchievements();
      setSteamSyncMessage(result.message);
      await reloadAchievementState();
    } catch (unknownError) {
      setSteamSyncError(errorMessage(unknownError));
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
      iconCache.clear();
      inflightIconFetches.clear();
      setIconVersion((v) => v + 1);
    } catch (unknownError) {
      setIconSyncError(errorMessage(unknownError));
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
        await reloadAchievementState();
      }
    } catch (unknownError) {
      setSyncError(errorMessage(unknownError));
    } finally {
      setSyncing(false);
    }
  };

  const reloadAchievementState = async () => {
    const [ach, cat, ovr, latestScanReport] = await Promise.all([
      loadAchievements(),
      loadCatalogInfo(),
      loadCompletionOverrides(),
      scanLocalStateCached({ force: true }),
    ]);
    setAchievements(ach);
    setCatalogInfo(cat);
    setOverrides(toOverrideRecord(ovr));
    setScanReport(latestScanReport);
  };

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const existing = getCachedScanReport();
        const [ach, cat, ovr, latestScanReport] = await Promise.all([
          loadAchievements(),
          loadCatalogInfo(),
          loadCompletionOverrides(),
          existing ? Promise.resolve(existing) : scanLocalStateCached(),
        ]);
        if (!cancelled) {
          setAchievements(ach);
          setCatalogInfo(cat);
          setOverrides(toOverrideRecord(ovr));
          setScanReport(latestScanReport);
        }
      } catch (unknownError) {
        if (!cancelled) {
          setError(errorMessage(unknownError));
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

  const dlcStatusByGroup = useMemo(
    () => buildDlcStatusByGroup(scanReport?.documents?.launcher?.dlcs ?? []),
    [scanReport],
  );

  const filtered = useMemo(() => {
    const q = search.toLowerCase().trim();
    return achievements
      .filter((a) => {
        const searchable = [
          a.source.name,
          a.source.requirement,
          a.source.description,
          a.source.group,
          a.steam_api_name,
          ...a.curation.tags,
        ]
          .filter(Boolean)
          .join(' ')
          .toLowerCase();
        if (q && !searchable.includes(q)) return false;
        if (completionFilter === 'completed' && !isCompleted(a, overrides)) return false;
        if (completionFilter === 'incomplete' && isCompleted(a, overrides)) return false;
        if (groupFilter !== 'All' && a.source.group !== groupFilter) return false;
        const dlcStatus = getAchievementDlcStatus(a, dlcStatusByGroup);
        if (dlcAvailabilityFilter === 'attention' && dlcStatus !== 'attention') return false;
        if (dlcAvailabilityFilter === 'available' && dlcStatus !== 'available') return false;
        if (dlcAvailabilityFilter === 'unknown' && dlcStatus !== 'unknown') return false;
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
        }
        return sortDir === 'asc' ? cmp : -cmp;
      });
  }, [achievements, search, completionFilter, groupFilter, dlcAvailabilityFilter, difficultyFilter, sortKey, sortDir, overrides, dlcStatusByGroup]);

  useEffect(() => {
    if (filtered.length === 0) {
      setSelectedId(null);
      return;
    }
    if (selectedId && !filtered.some((achievement) => achievement.id === selectedId)) {
      setSelectedId(null);
    }
  }, [filtered, selectedId]);

  const selectedAchievement = useMemo(
    () => filtered.find((achievement) => achievement.id === selectedId) ?? null,
    [filtered, selectedId],
  );
  const completedCount = achievements.filter((achievement) => isCompleted(achievement, overrides)).length;
  const warningCount = achievements.filter((achievement) => achievement.curation.warnings.length > 0).length;

  const handleCompletionToggle = async (id: string) => {
    setOverrideError(null);
    const prevOverride = id in overrides ? overrides[id] : undefined;
    const achievement = achievements.find((entry) => entry.id === id);

    let newOverride: boolean | undefined;
    if (prevOverride !== undefined) {
      // Has manual override (true or false) -> clear it
      newOverride = undefined;
    } else if (achievement?.completed) {
      // Steam baseline complete, no override -> force incomplete locally
      newOverride = false;
    } else {
      // No override and not completed -> mark complete locally
      newOverride = true;
    }

    // Optimistic update
    setOverrides((prev) => {
      const next = { ...prev };
      if (newOverride === undefined) {
        delete next[id];
      } else {
        next[id] = newOverride;
      }
      return next;
    });

    try {
      if (newOverride === undefined) {
        await clearCompletionOverride(id);
      } else {
        await setCompletionOverride(id, newOverride);
      }
    } catch (unknownError) {
      setOverrideError(errorMessage(unknownError));
      // Rollback only this id
      setOverrides((prev) => {
        const next = { ...prev };
        if (prevOverride === undefined) {
          delete next[id];
        } else {
          next[id] = prevOverride;
        }
        return next;
      });
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

  const toggleColumn = (key: ColumnKey) => {
    setVisibleColumns((current) => ({ ...current, [key]: !current[key] }));
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
        <h2>Achievement Catalog</h2>
        <p role="alert" className="error">
          {error}
        </p>
      </section>
    );
  }

  return (
    <section className="panel achievements-command-panel">
      <div className="panel-header achievements-header">
        <div>
          <p className="eyebrow">Achievement Operations</p>
          <h2>Achievement Catalog</h2>
          <p className="muted panel-subtitle">
            {achievements.length} records · {filtered.length} shown · {completedCount} completed · {warningCount} with warnings
          </p>
        </div>
        <ViewToggle viewMode={viewMode} onChange={setViewMode} />
      </div>
      <p className="sr-only" aria-live="polite">
        {viewMode === 'list'
          ? 'List view active. Selected achievement details are available.'
          : 'Board view active. Selecting a board card opens it in list view.'}
      </p>

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

      <StatusMessages
        messages={[syncMessage, steamSyncMessage, iconSyncMessage]}
        errors={[
          syncError ? `Sync error: ${syncError}` : null,
          steamSyncError ? `Steam sync error: ${steamSyncError}` : null,
          iconSyncError ? `Icon sync error: ${iconSyncError}` : null,
          overrideError ? `Override error: ${overrideError}` : null,
        ]}
      />

      <div className="achievement-filter-panel" aria-label="Achievement filters">
        <label className="achievement-filter-field achievement-filter-search">
          <span>Search</span>
          <input
            type="search"
            aria-label="Search achievements"
            placeholder="Name, requirement, tag, DLC…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="filter-input achievement-search"
          />
        </label>
        <label className="achievement-filter-field">
          <span>Status</span>
          <select
            value={completionFilter}
            onChange={(e) => setCompletionFilter(e.target.value as CompletionFilter)}
            className="filter-select"
            aria-label="Filter by completion status"
          >
            <option value="all">All achievements</option>
            <option value="completed">Completed only</option>
            <option value="incomplete">Incomplete only</option>
          </select>
        </label>
        <label className="achievement-filter-field">
          <span>DLC</span>
          <select
            value={groupFilter}
            onChange={(e) => setGroupFilter(e.target.value)}
            className="filter-select"
            aria-label="Filter by DLC"
          >
            {groups.map((g) => (
              <option key={g} value={g}>
                {g === 'All' ? 'All DLC' : g}
              </option>
            ))}
          </select>
        </label>
        <label className="achievement-filter-field">
          <span>DLC status</span>
          <select
            value={dlcAvailabilityFilter}
            onChange={(e) => setDlcAvailabilityFilter(e.target.value as DlcAvailabilityFilter)}
            className="filter-select"
            aria-label="Filter by DLC availability"
          >
            <option value="all">All statuses</option>
            <option value="attention">Needs DLC attention</option>
            <option value="available">Locally enabled</option>
            <option value="unknown">Unknown local status</option>
          </select>
        </label>
        <label className="achievement-filter-field">
          <span>Difficulty</span>
          <select
            value={difficultyFilter}
            onChange={(e) => setDifficultyFilter(e.target.value)}
            className="filter-select"
            aria-label="Filter by difficulty"
          >
            {DIFFICULTIES.map((d) => (
              <option key={d} value={d}>
                {d === 'All' ? 'All difficulties' : `${d} · ${DIFFICULTY_LABELS[d]}`}
              </option>
            ))}
          </select>
        </label>
        <label className="achievement-filter-field">
          <span>Sort</span>
          <select
            value={sortKey}
            onChange={(event) => handleSort(event.target.value as SortKey)}
            className="filter-select compact-select"
            aria-label="Sort achievements"
          >
            <option value="name">Name{sortArrow('name')}</option>
            <option value="group">DLC{sortArrow('group')}</option>
            <option value="difficulty">Difficulty{sortArrow('difficulty')}</option>
          </select>
        </label>
        <div className="achievement-filter-actions">
          <button type="button" className="secondary-button" onClick={() => handleSort(sortKey)}>
            {sortDir === 'asc' ? 'Asc' : 'Desc'}
          </button>
          <button type="button" className="secondary-button" onClick={() => setShowColumns((value) => !value)}>
            Columns
          </button>
        </div>
      </div>

      {showColumns ? (
        <ColumnControls
          visibleColumns={visibleColumns}
          onToggle={toggleColumn}
          onReset={resetColumns}
          changed={columnsChanged}
        />
      ) : null}

      <DifficultyLegend />

      {filtered.length === 0 ? (
        <p className="muted empty-state">No achievements match the current filters.</p>
      ) : viewMode === 'list' ? (
        <AchievementSplitView
          achievements={filtered}
          selectedAchievement={selectedAchievement}
          selectedId={selectedId}
          visibleColumns={visibleColumns}
          overrides={overrides}
          iconVersion={iconVersion}
          dlcStatusByGroup={dlcStatusByGroup}
          onSelect={setSelectedId}
          onCompletionToggle={handleCompletionToggle}
        />
      ) : (
        <AchievementBoardView
          achievements={filtered}
          overrides={overrides}
          iconVersion={iconVersion}
          dlcStatusByGroup={dlcStatusByGroup}
          onSelect={(achievement) => {
            setSelectedId(achievement.id);
            setViewMode('list');
          }}
          onCompletionToggle={handleCompletionToggle}
        />
      )}
    </section>
  );
}

function ViewToggle({
  viewMode,
  onChange,
}: {
  viewMode: ViewMode;
  onChange: (viewMode: ViewMode) => void;
}) {
  return (
    <div className="achievement-view-toggle" aria-label="Achievement view mode">
      <button
        type="button"
        className={viewMode === 'list' ? 'active' : ''}
        aria-pressed={viewMode === 'list'}
        onClick={() => onChange('list')}
      >
        List
      </button>
      <button
        type="button"
        className={viewMode === 'board' ? 'active' : ''}
        aria-pressed={viewMode === 'board'}
        onClick={() => onChange('board')}
      >
        Board
      </button>
    </div>
  );
}

function ColumnControls({
  visibleColumns,
  onToggle,
  onReset,
  changed,
}: {
  visibleColumns: Record<ColumnKey, boolean>;
  onToggle: (key: ColumnKey) => void;
  onReset: () => void;
  changed: boolean;
}) {
  return (
    <div className="achievement-column-controls" aria-label="Visible achievement fields">
      <div>
        <strong>Visible fields</strong>
        <p className="muted">Show or hide list-view fields. Tags and Rule Confidence start hidden by default.</p>
      </div>
      <div className="achievement-column-grid">
        {COLUMN_CONFIG.map((column) => (
          <label key={column.key} className="column-toggle">
            <input
              type="checkbox"
              checked={visibleColumns[column.key]}
              onChange={() => onToggle(column.key)}
            />
            <span>{column.label}</span>
          </label>
        ))}
      </div>
      <button type="button" className="secondary-button" onClick={onReset} disabled={!changed}>
        Reset
      </button>
    </div>
  );
}

function DifficultyLegend() {
  return (
    <div className="difficulty-legend" aria-label="Difficulty legend">
      <strong>Difficulty</strong>
      {Object.entries(DIFFICULTY_LABELS).map(([key, label]) => (
        <span key={key} className={`difficulty-legend-chip ${difficultyTone(key)}`}>
          <b>{key}</b> - {label}
        </span>
      ))}
    </div>
  );
}

function StatusMessages({ messages, errors }: { messages: Array<string | null>; errors: Array<string | null> }) {
  return (
    <>
      {messages.filter(Boolean).map((message, index) => (
        <p key={`message-${index}-${message}`} className="sync-message">
          {message}
        </p>
      ))}
      {errors.filter(Boolean).map((error, index) => (
        <p key={`error-${index}-${error}`} role="alert" className="error achievement-error">
          {error}
        </p>
      ))}
    </>
  );
}

function AchievementSplitView({
  achievements,
  selectedAchievement,
  selectedId,
  visibleColumns,
  overrides,
  iconVersion,
  dlcStatusByGroup,
  onSelect,
  onCompletionToggle,
}: {
  achievements: AchievementEntry[];
  selectedAchievement: AchievementEntry | null;
  selectedId: string | null;
  visibleColumns: Record<ColumnKey, boolean>;
  overrides: Record<string, boolean>;
  iconVersion: number;
  dlcStatusByGroup: Map<string, LocalDlcStatus>;
  onSelect: (id: string | null) => void;
  onCompletionToggle: (id: string) => void;
}) {
  return (
    <div className="achievement-split-layout">
      <div className="achievement-list-panel" aria-label="Achievement list">
        <div className="achievement-list-head">
          <strong>{achievements.length} records</strong>
          <span className="muted">Tactical split list</span>
        </div>
        <div className="achievement-split-list">
          {achievements.map((achievement) => (
            <AchievementListRow
              key={achievement.id}
              achievement={achievement}
              selected={achievement.id === selectedId}
              visibleColumns={visibleColumns}
              completed={isCompleted(achievement, overrides)}
              completionSource={completionSource(achievement, overrides)}
              iconVersion={iconVersion}
              dlcStatus={getAchievementDlcStatus(achievement, dlcStatusByGroup)}
              onSelect={() => onSelect(achievement.id)}
              onCompletionToggle={() => onCompletionToggle(achievement.id)}
            />
          ))}
        </div>
      </div>
      {selectedAchievement ? (
        <AchievementDetailPanel
          achievement={selectedAchievement}
          completed={isCompleted(selectedAchievement, overrides)}
          completionSource={completionSource(selectedAchievement, overrides)}
          iconVersion={iconVersion}
          dlcStatus={getAchievementDlcStatus(selectedAchievement, dlcStatusByGroup)}
          onCompletionToggle={() => onCompletionToggle(selectedAchievement.id)}
          onClose={() => onSelect(null)}
        />
      ) : null}
    </div>
  );
}

function AchievementListRow({
  achievement,
  selected,
  visibleColumns,
  completed,
  completionSource,
  iconVersion,
  dlcStatus,
  onSelect,
  onCompletionToggle,
}: {
  achievement: AchievementEntry;
  selected: boolean;
  visibleColumns: Record<ColumnKey, boolean>;
  completed: boolean;
  completionSource: string;
  iconVersion: number;
  dlcStatus: LocalDlcStatus;
  onSelect: () => void;
  onCompletionToggle: () => void;
}) {
  const tags = achievement.curation.tags.slice(0, 3);
  return (
    <article className={selected ? 'achievement-split-row active' : 'achievement-split-row'}>
      {visibleColumns.completion ? (
        <CompletionControl
          completed={completed}
          source={completionSource}
          onToggle={onCompletionToggle}
          label={completionToggleLabel(completionSource, achievement.source.name)}
        />
      ) : null}
      {visibleColumns.icon ? (
        <AchievementIcon achievementId={achievement.id} iconVersion={iconVersion} size={64} completed={completed} />
      ) : null}
      <button type="button" className="achievement-row-main" onClick={onSelect} aria-pressed={selected}>
        <span className="achievement-row-name">{achievement.source.name}</span>
        <span className="achievement-row-requirement">
          {achievement.source.requirement ?? achievement.source.description ?? 'No requirement text.'}
        </span>
        <span className="achievement-row-metadata">
          {visibleColumns.group ? <span>{achievement.source.group ?? 'No group'}</span> : null}
          {visibleColumns.version ? <span>v{achievement.source.version_added ?? '—'}</span> : null}
          {visibleColumns.steamApi ? <span>{achievement.steam_api_name ?? 'No Steam API name'}</span> : null}
        </span>
        {dlcStatus === 'attention' ? <span className="achievement-row-dlc-warning">DLC not enabled locally</span> : null}
      </button>
      <div className="achievement-row-signals">
        {dlcStatus === 'attention' ? <span className="badge badge-dlc-warning">DLC not enabled</span> : null}
        {visibleColumns.difficulty ? <DifficultyBadge difficulty={achievement.source.difficulty} /> : null}
        {visibleColumns.ruleConfidence && achievement.curation.rule_confidence ? (
          <span className="badge badge-unknown">{achievement.curation.rule_confidence}</span>
        ) : null}
        {visibleColumns.warnings && achievement.curation.warnings.length > 0 ? (
          <span className="badge badge-medium">{achievement.curation.warnings.length} warn</span>
        ) : null}
        {visibleColumns.tags && tags.length > 0 ? (
          <span className="achievement-row-tags">
            {tags.map((tag) => (
              <span key={tag} className="tag-pill">
                {tag}
              </span>
            ))}
          </span>
        ) : null}
      </div>
    </article>
  );
}

function AchievementDetailPanel({
  achievement,
  completed,
  completionSource,
  iconVersion,
  dlcStatus,
  onCompletionToggle,
  onClose,
}: {
  achievement: AchievementEntry;
  completed: boolean;
  completionSource: string;
  iconVersion: number;
  dlcStatus: LocalDlcStatus;
  onCompletionToggle: () => void;
  onClose: () => void;
}) {
  return (
    <aside className="achievement-detail-panel" aria-label="Selected achievement details">
      <div className="achievement-detail-art">
        <AchievementIcon achievementId={achievement.id} iconVersion={iconVersion} size={96} completed={completed} />
      </div>
      <div className="achievement-detail-body">
        <div className="achievement-detail-title-row">
          <div>
            <h3>{achievement.source.name}</h3>
            <p className="muted">{achievement.source.group ?? 'Uncategorized'} · {achievement.source.version_added ?? 'Unknown version'}</p>
          </div>
          <CompletionControl
            completed={completed}
            source={completionSource}
            onToggle={onCompletionToggle}
            label={completionToggleLabel(completionSource, achievement.source.name)}
            large
          />
          <button type="button" className="detail-close-button" onClick={onClose} aria-label="Close achievement details">
            Close
          </button>
        </div>
        <DetailSection title="Requirement" value={achievement.source.requirement} fallback="No requirement text." />
        <DetailSection title="Description" value={achievement.source.description} />
        <DetailSection title="Hint" value={achievement.source.hint} />
        {achievement.curation.planner_notes ? (
          <DetailSection title="Planner Notes" value={achievement.curation.planner_notes} />
        ) : null}
        <div className="achievement-fact-grid">
          <FactTile label="Difficulty" value={`${achievement.source.difficulty ?? 'UC'} · ${difficultyLabel(achievement.source.difficulty)}`} />
          <FactTile label="Rule Confidence" value={achievement.curation.rule_confidence ?? 'Unknown'} />
          <FactTile label="Steam API" value={achievement.steam_api_name ?? 'Unmapped'} />
          <FactTile label="Completion" value={completionSource} />
          <FactTile label="DLC status" value={dlcStatusLabel(dlcStatus)} />
        </div>
        {dlcStatus === 'attention' ? (
          <p className="detail-warning">This achievement's DLC group is currently disabled in the local launcher playset.</p>
        ) : null}
        {achievement.curation.tags.length > 0 ? (
          <div className="achievement-detail-tags">
            {achievement.curation.tags.map((tag) => (
              <span key={tag} className="tag-pill">
                {tag}
              </span>
            ))}
          </div>
        ) : null}
        {achievement.curation.warnings.length > 0 ? (
          <DetailList title="Warnings" items={achievement.curation.warnings} warning />
        ) : null}
        {achievement.curation.known_limitations.length > 0 ? (
          <DetailList title="Known Limitations" items={achievement.curation.known_limitations} />
        ) : null}
      </div>
    </aside>
  );
}

function AchievementBoardView({
  achievements,
  overrides,
  iconVersion,
  dlcStatusByGroup,
  onSelect,
  onCompletionToggle,
}: {
  achievements: AchievementEntry[];
  overrides: Record<string, boolean>;
  iconVersion: number;
  dlcStatusByGroup: Map<string, LocalDlcStatus>;
  onSelect: (achievement: AchievementEntry) => void;
  onCompletionToggle: (id: string) => void;
}) {
  const [expandedLanes, setExpandedLanes] = useState<Set<string>>(new Set());
  const lanes = useMemo(() => buildBoardLanes(achievements, overrides), [achievements, overrides]);
  return (
    <div className="achievement-board" aria-label="Achievement board view">
      {lanes.map((lane) => (
        <section key={lane.key} className="achievement-board-lane" aria-label={lane.title}>
          <div className="achievement-board-lane-head">
            <h3>{lane.title}</h3>
            <span className="muted">{lane.items.length}</span>
          </div>
          <div className="achievement-board-stack">
            {lane.items.length === 0 ? <p className="muted">No records in this lane.</p> : null}
            {(expandedLanes.has(lane.key) ? lane.items : lane.items.slice(0, 24)).map((achievement) => (
              <article key={achievement.id} className="achievement-board-card">
                {getAchievementDlcStatus(achievement, dlcStatusByGroup) === 'attention' ? (
                  <span className="board-card-dlc-flag">DLC not enabled</span>
                ) : null}
                <div className="achievement-board-card-top">
                  <AchievementIcon achievementId={achievement.id} iconVersion={iconVersion} size={56} completed={isCompleted(achievement, overrides)} />
                  <button type="button" className="achievement-board-title" onClick={() => onSelect(achievement)}>
                    <strong>{achievement.source.name}</strong>
                    <span>{achievement.source.requirement ?? achievement.source.description ?? 'No requirement text.'}</span>
                  </button>
                  <CompletionControl
                    completed={isCompleted(achievement, overrides)}
                    source={completionSource(achievement, overrides)}
                    onToggle={() => onCompletionToggle(achievement.id)}
                    label={completionToggleLabel(completionSource(achievement, overrides), achievement.source.name)}
                  />
                </div>
                <div className="planner-meta">
                  <DifficultyBadge difficulty={achievement.source.difficulty} />
                  {achievement.source.group ? <span className="tag-pill">{achievement.source.group}</span> : null}
                  {achievement.curation.rule_confidence ? (
                    <span className="tag-pill">{achievement.curation.rule_confidence}</span>
                  ) : null}
                </div>
              </article>
            ))}
            {lane.items.length > 24 ? (
              <button
                type="button"
                className="link-button board-lane-toggle"
                onClick={() => {
                  setExpandedLanes((current) => {
                    const next = new Set(current);
                    if (next.has(lane.key)) next.delete(lane.key);
                    else next.add(lane.key);
                    return next;
                  });
                }}
              >
                {expandedLanes.has(lane.key)
                  ? 'Show fewer'
                  : `Show all ${lane.items.length}`}
              </button>
            ) : null}
          </div>
        </section>
      ))}
    </div>
  );
}

function CompletionControl({
  completed,
  source,
  onToggle,
  label,
  large = false,
}: {
  completed: boolean;
  source: string;
  onToggle: () => void;
  label: string;
  large?: boolean;
}) {
  return (
    <button
      type="button"
      className={large ? 'completion-control large' : 'completion-control'}
      data-completed={completed ? 'true' : 'false'}
      aria-pressed={completed}
      aria-label={label}
      title={`${source}. ${label}`}
      onClick={(event) => {
        event.stopPropagation();
        onToggle();
      }}
    >
      {completed ? '\u2713' : '\u00B7'}
    </button>
  );
}

function AchievementIcon({
  achievementId,
  iconVersion,
  size = 56,
  completed,
}: {
  achievementId: string;
  iconVersion: number;
  size?: number;
  completed?: boolean;
}) {
  const cacheKey = `${achievementId}-${iconVersion}`;
  const [src, setSrc] = useState<string | null>(() => iconCache.get(cacheKey) ?? null);

  useEffect(() => {
    const cached = iconCache.get(cacheKey);
    if (cached !== undefined) {
      setSrc(cached);
      return;
    }

    let cancelled = false;
    loadAchievementIconDataUrl(achievementId, cacheKey).then((dataUri) => {
      if (!cancelled) setSrc(dataUri);
    });

    return () => {
      cancelled = true;
    };
  }, [achievementId, iconVersion, cacheKey]);

  const dataCompleted = completed ? 'true' : 'false';

  return src ? (
    <img className="achievement-icon" src={src} alt="" width={size} height={size} data-completed={dataCompleted} />
  ) : (
    <span className="achievement-icon placeholder" style={{ width: size, height: size }} data-completed={dataCompleted}>
      <IconPlaceholder size={Math.max(32, Math.round(size * 0.72))} />
    </span>
  );
}

function loadAchievementIconDataUrl(achievementId: string, cacheKey: string): Promise<string | null> {
  const cached = iconCache.get(cacheKey);
  if (cached !== undefined) return Promise.resolve(cached);

  const inflight = inflightIconFetches.get(cacheKey);
  if (inflight) return inflight;

  const fetchPromise = getAchievementIcon(achievementId)
    .then((bytes) => {
      let dataUri: string | null = null;
      if (bytes) {
        let binary = '';
        for (let i = 0; i < bytes.length; i++) {
          binary += String.fromCharCode(bytes[i]);
        }
        dataUri = 'data:image/png;base64,' + btoa(binary);
      }
      iconCache.set(cacheKey, dataUri);
      return dataUri;
    })
    .catch(() => {
      iconCache.set(cacheKey, null);
      return null;
    })
    .finally(() => {
      inflightIconFetches.delete(cacheKey);
    });

  inflightIconFetches.set(cacheKey, fetchPromise);
  return fetchPromise;
}

function DifficultyBadge({ difficulty }: { difficulty: string | null }) {
  return <span className={diffClass(difficulty)}>{difficulty ?? 'UC'}</span>;
}

function DetailSection({ title, value, fallback }: { title: string; value: string | null; fallback?: string }) {
  if (!value && !fallback) return null;
  return (
    <section className="achievement-detail-section">
      <h4>{title}</h4>
      <p>{value ?? fallback}</p>
    </section>
  );
}

function DetailList({ title, items, warning = false }: { title: string; items: string[]; warning?: boolean }) {
  return (
    <section className={warning ? 'achievement-detail-section warning' : 'achievement-detail-section'}>
      <h4>{title}</h4>
      <ul>
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </section>
  );
}

function FactTile({ label, value }: { label: string; value: string }) {
  return (
    <div className="achievement-fact-tile">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

/**
 * Maps achievement IDs to local override state:
 * - true: locally marked complete
 * - false: locally marked incomplete, overriding a true Steam baseline
 * - missing key: defer to Steam baseline
 */
function toOverrideRecord(overrides: AchievementOverride[]): Record<string, boolean> {
  const record: Record<string, boolean> = {};
  for (const o of overrides) {
    record[o.achievement_id] = o.completed;
  }
  return record;
}

function buildBoardLanes(achievements: AchievementEntry[], overrides: Record<string, boolean>) {
  const completed: AchievementEntry[] = [];
  const warnings: AchievementEntry[] = [];
  const lowFriction: AchievementEntry[] = [];
  const standard: AchievementEntry[] = [];
  const extreme: AchievementEntry[] = [];

  for (const achievement of achievements) {
    if (isCompleted(achievement, overrides)) {
      completed.push(achievement);
      continue;
    }
    if (achievement.curation.warnings.length > 0 || achievement.curation.known_limitations.length > 0) {
      warnings.push(achievement);
      continue;
    }
    const difficulty = achievement.source.difficulty ?? 'UC';
    if (difficulty === 'VE' || difficulty === 'E') lowFriction.push(achievement);
    else if (difficulty === 'M' || difficulty === 'H') standard.push(achievement);
    else extreme.push(achievement);
  }

  return [
    { key: 'low', title: 'Low Friction', items: lowFriction },
    { key: 'standard', title: 'Standard Ops', items: standard },
    { key: 'warnings', title: 'Needs Evidence', items: warnings },
    { key: 'extreme', title: 'Extreme / Unclear', items: extreme },
    { key: 'completed', title: 'Completed', items: completed },
  ];
}

function isCompleted(achievement: AchievementEntry, overrides: Record<string, boolean>) {
  return overrides[achievement.id] ?? achievement.completed ?? false;
}

function completionSource(achievement: AchievementEntry, overrides: Record<string, boolean>) {
  if (achievement.id in overrides) return 'Local override';
  if (achievement.completed) return 'Steam baseline';
  return 'Incomplete';
}

function completionToggleLabel(source: string, achievementName: string) {
  if (source === 'Local override') return `Clear local completion mark for ${achievementName}`;
  if (source === 'Steam baseline') return `Set local incomplete for ${achievementName}`;
  return `Mark locally completed: ${achievementName}`;
}

function diffClass(difficulty: string | null): string {
  const diff = difficulty ?? 'UC';
  if (diff === 'VE' || diff === 'E') return 'badge badge-easy';
  if (diff === 'M') return 'badge badge-medium';
  if (diff === 'H' || diff === 'VH') return 'badge badge-hard';
  return 'badge badge-insane';
}

function difficultyLabel(difficulty: string | null) {
  return DIFFICULTY_LABELS[difficulty ?? 'UC'] ?? DIFFICULTY_LABELS.UC;
}

function difficultyTone(difficulty: string) {
  if (difficulty === 'VE' || difficulty === 'E') return 'easy';
  if (difficulty === 'M') return 'medium';
  if (difficulty === 'H' || difficulty === 'VH') return 'hard';
  return 'insane';
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

type LocalDlcStatus = 'available' | 'attention' | 'unknown';

function buildDlcStatusByGroup(dlcs: LauncherDlcSummary[]): Map<string, LocalDlcStatus> {
  const statusByGroup = new Map<string, LocalDlcStatus>();
  for (const dlc of dlcs) {
    const name = dlc.name?.trim();
    if (!name) continue;
    const normalized = normalizeDlcLabel(name);
    if (!normalized) continue;
    const status: LocalDlcStatus =
      dlc.enabled_in_active_playset === true
        ? 'available'
        : dlc.enabled_in_active_playset === false
          ? 'attention'
          : 'unknown';
    statusByGroup.set(normalized, status);
  }
  return statusByGroup;
}

function getAchievementDlcStatus(
  achievement: AchievementEntry,
  statusByGroup: Map<string, LocalDlcStatus>,
): LocalDlcStatus {
  const group = achievement.source.group;
  if (!group || isBaseGameGroup(group)) return 'available';
  return statusByGroup.get(normalizeDlcLabel(group)) ?? 'unknown';
}

function normalizeDlcLabel(label: string): string {
  return label
    .toLowerCase()
    .replace(/\b(story|species|portrait|expansion|pack)\b/g, '')
    .replace(/[^a-z0-9]+/g, ' ')
    .trim()
    .replace(/\s+/g, ' ');
}

function isBaseGameGroup(group: string): boolean {
  return normalizeDlcLabel(group) === 'base game';
}

function dlcStatusLabel(status: LocalDlcStatus): string {
  if (status === 'available') return 'Enabled locally';
  if (status === 'attention') return 'Disabled in active playset';
  return 'Unknown';
}
