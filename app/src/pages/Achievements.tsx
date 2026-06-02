import { useEffect, useMemo, useState } from 'react';

import {
  loadAchievements,
  loadCatalogInfo,
  type AchievementEntry,
  type CatalogInfo,
} from '../tauri';

const DIFFICULTIES = ['All', 'VE', 'E', 'M', 'H', 'VH', 'I', 'UC'] as const;
type SortKey = 'name' | 'group' | 'difficulty' | 'version';
type SortDir = 'asc' | 'desc';

export function Achievements() {
  const [achievements, setAchievements] = useState<AchievementEntry[]>([]);
  const [catalogInfo, setCatalogInfo] = useState<CatalogInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

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
        const [ach, cat] = await Promise.all([loadAchievements(), loadCatalogInfo()]);
        if (!cancelled) {
          setAchievements(ach);
          setCatalogInfo(cat);
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
        <p className="catalog-info muted">
          Catalog v{catalogInfo.catalog_version}
          {catalogInfo.stellaris_version ? ` · Stellaris ${catalogInfo.stellaris_version}` : ''}
          {catalogInfo.updated_at ? ` · Updated ${catalogInfo.updated_at}` : ''}
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

      {filtered.length === 0 ? (
        <p className="muted">No achievements match the current filters.</p>
      ) : (
        <table className="achievement-table">
          <thead>
            <tr>
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
}

function AchievementRow({ ach, expanded, onToggle, diffClass }: AchievementRowProps) {
  return (
    <>
      <tr className="achievement-row" onClick={onToggle} role="button" tabIndex={0} onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onToggle(); } }}>
        <td>{ach.source.name}</td>
        <td>{ach.source.group ?? '—'}</td>
        <td>
          <span className={diffClass(ach.source.difficulty)}>
            {ach.source.difficulty ?? 'UC'}
          </span>
        </td>
        <td>{ach.source.version_added ?? '—'}</td>
      </tr>
      {expanded ? (
        <tr className="achievement-detail-row">
          <td colSpan={4}>
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
