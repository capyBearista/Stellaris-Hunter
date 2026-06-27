import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { beforeEach, expect, it, vi } from 'vitest';

const invoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({
  invoke,
}));

import { App } from './App';
import { Overview } from './pages/Overview';
import { Achievements } from './pages/Achievements';
import { Notes } from './pages/Notes';
import { Planner } from './pages/Planner';
import { Runs } from './pages/Runs';
import { invalidateScanCache, resetScanCache, scanLocalStateCached } from './scanCache';
import type { PersistedRunSummary, RunFactSummary } from './tauri';

beforeEach(() => {
  resetScanCache();
  invoke.mockReset();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve(null);
    }
    if (command === 'load_achievements') {
      return Promise.resolve([]);
    }
    if (command === 'load_app_info') {
      return Promise.resolve({
        appVersion: '1.0.0',
        catalogVersion: null,
        stellarisVersion: null,
        lastCatalogSync: null,
        lastSteamSync: null,
        lastSteamSyncStatus: null,
        lastSteamSyncError: null,
        lastSaveScan: null,
      });
    }
    if (command === 'load_planner_status_counts') {
      return Promise.resolve({
        completed: 0,
        planned: 0,
        possible: 0,
        incompatible: 0,
        impossible: 0,
        unknown: 0,
        incomplete: 0,
      });
    }
    if (command === 'load_run_facts') {
      return Promise.resolve([]);
    }
    if (command === 'load_planner_evaluations') {
      return Promise.resolve([]);
    }
    if (command === 'set_run_achievement_status') {
      return Promise.resolve();
    }
    if (command === 'load_run_notes') {
      return Promise.resolve(null);
    }
    if (command === 'load_run_achievement_notes') {
      return Promise.resolve([]);
    }
    if (command === 'rescan_saves') {
      return Promise.resolve([]);
    }
    if (command === 'reparse_run_save') {
      return Promise.resolve(null);
    }
    if (command === 'scan_local_state') {
      return Promise.resolve({ errors: [] });
    }
    return Promise.resolve(null);
  });
});

it('renders the app shell with navigation links', async () => {
  render(<App />);

  expect(screen.getAllByText('Stellaris Hunter').length).toBeGreaterThanOrEqual(1);
  expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /achievements/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /planner/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /notes/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /runs/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /settings/i })).toBeInTheDocument();
  expect(await screen.findByRole('button', { name: /rescan saves/i })).toBeInTheDocument();
});

it('renders notes page with heading and empty state', async () => {
  const user = userEvent.setup();
  render(<App />);

  const notesLink = screen.getByRole('link', { name: /notes/i });
  expect(notesLink).toBeInTheDocument();

  await user.click(notesLink);

  expect(await screen.findByRole('heading', { name: /notes/i })).toBeInTheDocument();
  expect(screen.getByText(/no notes found/i)).toBeInTheDocument();
});

it('renders overview page with heading and scan button', async () => {
  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  expect(screen.getByRole('heading', { name: /overview/i })).toBeInTheDocument();
  expect(await screen.findByRole('button', { name: /rescan saves/i })).toBeInTheDocument();
});

it('loads persisted overview state and rescans saves', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve({
        catalog_version: '1.0.1',
        stellaris_version: '4.0',
        source_url: null,
        source_hash: null,
        updated_at: '2026-06-03',
        imported_at: '2026-06-03',
      });
    }
    if (command === 'load_achievements') {
      return Promise.resolve([{ id: 'ach_1' }, { id: 'ach_2' }]);
    }
    if (command === 'rescan_saves') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 12,
          updated_at: '2026-06-03',
        },
      ]);
    }
    return Promise.resolve([]);
  });

  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  expect(await screen.findByText('1.0.1')).toBeInTheDocument();
  expect(screen.getByText('2')).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: /rescan saves/i }));

  expect(await screen.findByText('Synthetic Run')).toBeInTheDocument();
  expect(screen.getByText('12 facts')).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith('rescan_saves', {});
});

it('shows steam sync info and planner status counts on overview', async () => {
  invoke.mockImplementation((command: string, args?: Record<string, unknown>) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 12,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve(null);
    }
    if (command === 'load_achievements') {
      return Promise.resolve([{ id: 'ach_1' }]);
    }
    if (command === 'load_app_info') {
      return Promise.resolve({
        appVersion: '1.0.0',
        catalogVersion: '1.0.1',
        stellarisVersion: '4.0',
        lastCatalogSync: null,
        lastSteamSync: '2026-06-04T12:34:56Z',
        lastSteamSyncStatus: 'failed',
        lastSteamSyncError: 'Steam client not running',
        lastSaveScan: null,
      });
    }
    if (command === 'load_planner_status_counts') {
      expect(args).toEqual({ runId: '/tmp/documents/save games/run_a' });
      return Promise.resolve({
        completed: 1,
        planned: 2,
        possible: 3,
        incompatible: 4,
        impossible: 5,
        unknown: 6,
        incomplete: 7,
      });
    }
    return Promise.resolve({ errors: [] });
  });

  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  expect(await screen.findByText('failed')).toBeInTheDocument();
  expect(screen.getByText('2026-06-04T12:34:56Z')).toBeInTheDocument();
  expect(screen.getByText(/steam client not running/i)).toBeInTheDocument();
  expect(screen.getByText('Completed')).toBeInTheDocument();
  expect(screen.getByText('1 achievements')).toBeInTheDocument();
  expect(screen.getByText('Incomplete')).toBeInTheDocument();
  expect(screen.getByText('7 achievements')).toBeInTheDocument();
});

it('uses the planner run id route param as the initial selection', async () => {
  invoke.mockImplementation((command: string, args?: Record<string, unknown>) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/runs/a',
          run_folder: 'run-a',
          display_name: 'Run A',
          latest_save_path: null,
          latest_save_file_name: null,
          latest_ingame_date: null,
          game_version: null,
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 2,
          updated_at: '2026-06-03',
        },
        {
          folder_path: '/runs/b',
          run_folder: 'run-b',
          display_name: 'Run B',
          latest_save_path: null,
          latest_save_file_name: null,
          latest_ingame_date: '2400.01.01',
          game_version: null,
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 4,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_planner_evaluations') {
      if (args?.runFolderPath === '/runs/b') {
        return Promise.resolve([
          {
            achievement: {
              id: 'ach_1',
              steam_app_id: 281990,
              steam_api_name: null,
              local_key: null,
              deprecated: false,
              source: { name: 'Planner Target', description: null, requirement: null, hint: null, group: null, version_added: null, difficulty: null },
              curation: { tags: [], conditions: [], warnings: [], planner_notes: null, known_limitations: [], rule_confidence: null },
            },
            status: 'Possible',
            computed_status: 'Possible',
            planned: false,
            ignored: false,
            reasons: [],
            warnings: [],
            conditions: [],
          },
        ]);
      }
      return Promise.resolve([]);
    }
    if (command === 'load_run_achievement_notes') {
      return Promise.resolve([]);
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter initialEntries={['/planner/%2Fruns%2Fb']}>
      <Routes>
        <Route path="/planner/:runId" element={<Planner />} />
      </Routes>
    </MemoryRouter>,
  );

  expect(await screen.findByText(/evaluating run b/i)).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith('load_planner_evaluations', { runFolderPath: '/runs/b' });
});

it('shows run eligibility and reparses the latest save from the runs page', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string, args?: Record<string, unknown>) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/runs/a',
          run_folder: 'run-a',
          display_name: 'Run A',
          latest_save_path: '/runs/a/latest.sav',
          latest_save_file_name: 'latest.sav',
          latest_ingame_date: '2400.01.01',
          game_version: '4.0',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 2,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_run_facts') {
      return Promise.resolve([{ run_folder_path: '/runs/a', dimension: 'empire', key: 'origin', value: 'origin_default', source: 'save', confidence: 'confirmed', updated_from_save_path: null, updated_at: '2026-06-03', is_override: false }]);
    }
    if (command === 'load_run_notes') {
      return Promise.resolve(null);
    }
    if (command === 'scan_local_state') {
      return Promise.resolve({
        errors: [],
        documents: {
          save_runs: [
            {
              run_folder: 'run-a',
              latest_save: { required_dlcs: ['Utopia'] },
              eligibility: {
                conclusion: 'likely_eligible',
                cheated_on_save: false,
                ironman: true,
                mod_risk: 'none',
                reasons: ['Ironman save detected'],
                warnings: ['Checksum data unavailable'],
              },
              dlc_info: {
                enabled_and_required: ['Utopia'],
                disabled_but_required: [],
                unknown_status_required: [],
                all_enabled_dlcs: ['Utopia'],
                all_disabled_dlcs: [],
              },
            },
          ],
        },
      });
    }
    if (command === 'reparse_run_save') {
      expect(args).toEqual({ runId: '/runs/a' });
      return Promise.resolve(null);
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );

  await user.click(await screen.findByRole('button', { name: /run a/i }));
  expect(await screen.findByText(/likely_eligible/i)).toBeInTheDocument();
  expect(screen.getByText(/ironman save detected/i)).toBeInTheDocument();
  expect(screen.getByText(/checksum data unavailable/i)).toBeInTheDocument();

  await user.click(screen.getByRole('button', { name: /parse latest save/i }));

  expect(invoke).toHaveBeenCalledWith('reparse_run_save', { runId: '/runs/a' });
  expect(screen.getByRole('link', { name: /open in planner/i })).toHaveAttribute('href', '/planner/%2Fruns%2Fa');
});

it('renders achievements page with mocked data', async () => {
  const user = userEvent.setup();
  invoke.mockResolvedValueOnce([
    {
      id: 'ach_1',
      steam_app_id: 281990,
      steam_api_name: 'ACH_ONE',
      local_key: null,
      deprecated: false,
      source: {
        name: 'First Achievement',
        description: 'Do the thing',
        requirement: 'Complete the thing',
        hint: 'Try doing the thing',
        group: 'Base Game',
        version_added: '1.0',
        difficulty: 'E',
      },
      curation: {
        tags: ['early'],
        conditions: [],
        warnings: [],
        planner_notes: null,
        known_limitations: [],
        rule_confidence: null,
      },
    },
  ]);
  invoke.mockResolvedValueOnce({
    catalog_version: '1.0',
    stellaris_version: '4.0',
    source_url: null,
    source_hash: null,
    updated_at: '2025-01-01',
    imported_at: '2025-01-02',
  });
  invoke.mockResolvedValueOnce([]); // loadCompletionOverrides

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  const achievementTitles = await screen.findAllByText('First Achievement');
  expect(achievementTitles.length).toBeGreaterThanOrEqual(1);
  expect(screen.getByLabelText('Achievement list')).toBeInTheDocument();
  expect(screen.queryByLabelText('Selected achievement details')).not.toBeInTheDocument();
  await user.click(achievementTitles[0]);
  expect(screen.getByLabelText('Selected achievement details')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: /Close achievement details/i }));
  expect(screen.queryByLabelText('Selected achievement details')).not.toBeInTheDocument();
  expect(screen.getAllByText('Base Game').length).toBeGreaterThanOrEqual(1);
});

it('invokes the scan command with an empty payload', async () => {
  invoke.mockResolvedValueOnce({ errors: [] });

  const { scanLocalState } = await import('./tauri');
  await scanLocalState();

  expect(invoke).toHaveBeenCalledWith('scan_local_state', {});
});

it('sets data-completed attribute on achievement icons per completion state', async () => {
  invoke.mockImplementation((command: string) => {
    if (command === 'load_achievements') {
      return Promise.resolve([
        {
          id: 'ach_done',
          steam_app_id: 281990,
          steam_api_name: 'ACH_DONE',
          local_key: null,
          deprecated: false,
          completed: true,
          source: {
            name: 'Done Achievement',
            description: null,
            requirement: 'Do the thing',
            hint: null,
            group: 'Base Game',
            version_added: '1.0',
            difficulty: 'E',
          },
          curation: {
            tags: [],
            conditions: [],
            warnings: [],
            planner_notes: null,
            known_limitations: [],
            rule_confidence: null,
          },
        },
        {
          id: 'ach_not_done',
          steam_app_id: 281990,
          steam_api_name: 'ACH_NOT',
          local_key: null,
          deprecated: false,
          completed: false,
          source: {
            name: 'Not Done Achievement',
            description: null,
            requirement: 'Do the other thing',
            hint: null,
            group: 'Base Game',
            version_added: '1.0',
            difficulty: 'M',
          },
          curation: {
            tags: [],
            conditions: [],
            warnings: [],
            planner_notes: null,
            known_limitations: [],
            rule_confidence: null,
          },
        },
      ]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve({
        catalog_version: '1.0',
        stellaris_version: '4.0',
        source_url: null,
        source_hash: null,
        updated_at: '2025-01-01',
        imported_at: '2025-01-02',
      });
    }
    if (command === 'load_completion_overrides') return Promise.resolve([]);
    if (command === 'scan_local_state') return Promise.resolve({ errors: [] });
    if (command === 'get_achievement_icon') return Promise.resolve(null);
    return Promise.resolve(null);
  });

  const { container } = render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  await screen.findByText('Done Achievement');

  const icons = container.querySelectorAll<HTMLElement>('.achievement-icon');
  expect(icons.length).toBeGreaterThanOrEqual(1);

  // Collect data-completed values from all icons
  const dataCompletedValues = Array.from(icons).map((el) => el.dataset.completed);
  expect(dataCompletedValues).toContain('true');
  expect(dataCompletedValues).toContain('false');
});

it('forces a fresh scan after cache invalidation', async () => {
  let scanCount = 0;
  invoke.mockImplementation((command: string) => {
    if (command === 'scan_local_state') {
      scanCount += 1;
      return Promise.resolve({ errors: [], scanCount });
    }
    return Promise.resolve(null);
  });

  const first = await scanLocalStateCached();
  const cached = await scanLocalStateCached();
  invalidateScanCache();
  const refreshed = await scanLocalStateCached();

  expect(first).toEqual({ errors: [], scanCount: 1 });
  expect(cached).toEqual({ errors: [], scanCount: 1 });
  expect(refreshed).toEqual({ errors: [], scanCount: 2 });
});

it('loads persisted runs and facts on the runs page', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((cmd: string) => {
    if (cmd === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 2,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (cmd === 'load_run_facts') {
      return Promise.resolve([
        {
          run_folder_path: '/tmp/documents/save games/run_a',
          dimension: 'empire',
          key: 'origin',
          value: 'origin_default',
          source: 'parsed_save',
          confidence: 'high',
          updated_from_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          updated_at: '2026-06-03',
        },
        ...Array.from({ length: 12 }, (_, index) => ({
          run_folder_path: '/tmp/documents/save games/run_a',
          dimension: 'test',
          key: `fact_${index}`,
          value: `value_${index}`,
          source: 'parsed_save',
          confidence: 'high',
          updated_from_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          updated_at: '2026-06-03',
        })),
      ]);
    }
    if (cmd === 'load_run_notes') return Promise.resolve(null);
    if (cmd === 'load_run_achievement_notes') return Promise.resolve([]);
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );

  expect((await screen.findAllByText('Synthetic Run')).length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText('ironman.sav')).toBeInTheDocument();
  expect(await screen.findByText('empire.origin')).toBeInTheDocument();
  expect(screen.getByText('origin_default')).toBeInTheDocument();
  expect(screen.queryByText('test.fact_11')).not.toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: /show all 13 facts/i }));
  expect(await screen.findByText('test.fact_11')).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /show fewer facts/i })).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith('load_runs', {});
  expect(invoke).toHaveBeenCalledWith('load_run_facts', {
    runFolderPath: '/tmp/documents/save games/run_a',
  });
});

it('rescans saves from the runs page', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((cmd: string) => {
    if (cmd === 'load_runs') return Promise.resolve([]);
    if (cmd === 'load_run_facts') return Promise.resolve([]);
    if (cmd === 'scan_local_state') return Promise.resolve({ errors: [] });
    if (cmd === 'rescan_saves') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_b',
          run_folder: 'run_b',
          display_name: null,
          latest_save_path: '/tmp/documents/save games/run_b/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: null,
          game_version: null,
          parse_status: 'failed',
          parse_error: 'failed to parse save',
          fact_count: 0,
          updated_at: '2026-06-03',
        },
      ]);
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );

  await screen.findByText(/no persisted runs yet/i);
  await user.click(screen.getByRole('button', { name: /rescan saves/i }));

  expect((await screen.findAllByText('run_b')).length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText('failed to parse save')).toBeInTheDocument();
  expect(invoke).toHaveBeenCalledWith('rescan_saves', {});
});

it('loads planner evaluations and toggles planned status', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 12,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_planner_evaluations') {
      return Promise.resolve([
        {
          achievement: {
            id: 'ach_1',
            steam_app_id: 281990,
            steam_api_name: 'ACH_ONE',
            local_key: null,
            deprecated: false,
            source: {
              name: 'First Achievement',
              description: null,
              requirement: 'Complete the thing',
              hint: null,
              group: 'Base Game',
              version_added: '1.0',
              difficulty: 'E',
            },
            curation: {
              tags: ['early'],
              conditions: [],
              warnings: [],
              planner_notes: null,
              known_limitations: [],
              rule_confidence: 'medium',
            },
          },
          status: 'Possible',
          computed_status: 'Possible',
          planned: false,
          ignored: false,
          reasons: ['No hard blocker is known.'],
          warnings: [],
          conditions: [],
        },
      ]);
    }
    if (command === 'set_run_achievement_status') {
      return Promise.resolve();
    }
    return Promise.resolve([]);
  });

  render(
    <MemoryRouter>
      <Planner />
    </MemoryRouter>,
  );

  expect(await screen.findByRole('heading', { name: /planner/i })).toBeInTheDocument();
  expect(await screen.findByText('First Achievement')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: /^plan$/i }));

  expect(invoke).toHaveBeenCalledWith('load_planner_evaluations', {
    runFolderPath: '/tmp/documents/save games/run_a',
  });
  expect(invoke).toHaveBeenCalledWith('set_run_achievement_status', {
    runFolderPath: '/tmp/documents/save games/run_a',
    achievementId: 'ach_1',
    userStatus: 'planned',
  });
});

it('surfaces planner toggle failures', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 12,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_planner_evaluations') {
      return Promise.resolve([
        {
          achievement: {
            id: 'ach_1',
            steam_app_id: 281990,
            steam_api_name: 'ACH_ONE',
            local_key: null,
            deprecated: false,
            source: {
              name: 'First Achievement',
              description: null,
              requirement: 'Complete the thing',
              hint: null,
              group: 'Base Game',
              version_added: '1.0',
              difficulty: 'E',
            },
            curation: {
              tags: ['early'],
              conditions: [],
              warnings: [],
              planner_notes: null,
              known_limitations: [],
              rule_confidence: 'medium',
            },
          },
          status: 'Possible',
          computed_status: 'Possible',
          planned: false,
          ignored: false,
          reasons: ['No hard blocker is known.'],
          warnings: [],
          conditions: [],
        },
      ]);
    }
    if (command === 'set_run_achievement_status') {
      return Promise.reject(new Error('planner write failed'));
    }
    return Promise.resolve([]);
  });

  render(
    <MemoryRouter>
      <Planner />
    </MemoryRouter>,
  );

  expect(await screen.findByText('First Achievement')).toBeInTheDocument();
  await user.click(screen.getByRole('button', { name: /^plan$/i }));
  expect(await screen.findByRole('alert')).toHaveTextContent('planner write failed');
});

it('shows advisory details for possible planner achievements with warnings', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 12,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_planner_evaluations') {
      return Promise.resolve([
        {
          achievement: {
            id: 'ach_1',
            steam_app_id: 281990,
            steam_api_name: 'ACH_ONE',
            local_key: null,
            deprecated: false,
            source: {
              name: 'First Achievement',
              description: null,
              requirement: 'Complete the thing',
              hint: null,
              group: 'Base Game',
              version_added: '1.0',
              difficulty: 'E',
            },
            curation: {
              tags: ['early'],
              conditions: [],
              warnings: [],
              planner_notes: null,
              known_limitations: [],
              rule_confidence: 'medium',
            },
          },
          status: 'Possible',
          computed_status: 'Possible',
          planned: false,
          ignored: false,
          reasons: ['No hard blocker is known.'],
          warnings: ['Requires some luck, the current facts are not a guarantee.'],
          conditions: [],
        },
      ]);
    }
    if (command === 'load_run_achievement_notes') return Promise.resolve([]);
    if (command === 'set_run_achievement_status') return Promise.resolve();
    return Promise.resolve([]);
  });

  render(
    <MemoryRouter>
      <Planner />
    </MemoryRouter>,
  );

  const title = await screen.findByText('First Achievement');
  const plannerCard = title.closest('.planner-item');
  expect(plannerCard).not.toBeNull();

  const card = within(plannerCard as HTMLElement);
  expect(card.getAllByText('Note')).toHaveLength(1);
  expect(card.queryByText('Warning')).not.toBeInTheDocument();

  await user.click(card.getByRole('button', { name: /details/i }));

  expect(
    await card.findByText('Requires some luck, the current facts are not a guarantee.'),
  ).toBeInTheDocument();
});

it('edits a fact override from the runs page', async () => {
  const user = userEvent.setup();
  const mockRun: PersistedRunSummary = {
    folder_path: '/test/run',
    run_folder: 'run',
    display_name: 'Test Run',
    latest_save_path: '/test/run/ironman.sav',
    latest_save_file_name: 'ironman.sav',
    latest_ingame_date: '2200.01.01',
    game_version: '4.3.0',
    parse_status: 'parsed',
    parse_error: null,
    fact_count: 1,
    updated_at: '2026-01-01',
  };
  const mockFact: RunFactSummary = {
    run_folder_path: '/test/run',
    dimension: 'empire',
    key: 'origin',
    value: 'origin_default',
    source: 'parsed_save',
    confidence: 'high',
    updated_from_save_path: '/test/run/ironman.sav',
    updated_at: '2026-01-01',
    is_override: false,
  };

  invoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'load_runs') return [mockRun];
    if (cmd === 'load_run_facts') return [mockFact];
    return null;
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );
  expect((await screen.findAllByText('Test Run')).length).toBeGreaterThanOrEqual(1);

  // Click the run to load facts
  await user.click(screen.getByRole('button', { name: /Test Run/i }));
  await screen.findByText('empire.origin');

  // Click Edit button
  await user.click(screen.getByRole('button', { name: 'Edit' }));

  // Fill in the edit form
  const valueInput = screen.getByPlaceholderText('Value (JSON)');
  await user.clear(valueInput);
  await user.type(valueInput, '"origin_synaptic"');

  const reasonInput = screen.getByPlaceholderText('Reason (optional)');
  await user.type(reasonInput, 'corrected');

  // Click Save
  await user.click(screen.getByRole('button', { name: 'Save' }));

  // Verify the IPC call
  expect(invoke).toHaveBeenCalledWith('set_fact_override', {
    runFolderPath: '/test/run',
    dimension: 'empire',
    key: 'origin',
    valueJson: '"origin_synaptic"',
    reason: 'corrected',
  });
});

it('clears a fact override from the runs page', async () => {
  const user = userEvent.setup();
  const mockRun: PersistedRunSummary = {
    folder_path: '/test/run',
    run_folder: 'run',
    display_name: 'Test Run',
    latest_save_path: '/test/run/ironman.sav',
    latest_save_file_name: 'ironman.sav',
    latest_ingame_date: '2200.01.01',
    game_version: '4.3.0',
    parse_status: 'parsed',
    parse_error: null,
    fact_count: 1,
    updated_at: '2026-01-01',
  };
  const mockFact: RunFactSummary = {
    run_folder_path: '/test/run',
    dimension: 'empire',
    key: 'origin',
    value: 'origin_synaptic',
    source: 'user_override',
    confidence: 'high',
    updated_from_save_path: null,
    updated_at: '2026-01-01',
    is_override: true,
  };

  invoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'load_runs') return [mockRun];
    if (cmd === 'load_run_facts') return [mockFact];
    return null;
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );
  expect((await screen.findAllByText('Test Run')).length).toBeGreaterThanOrEqual(1);

  // Click the run to load facts
  await user.click(screen.getByRole('button', { name: /Test Run/i }));
  await screen.findByText('override');

  // Click Clear override button
  await user.click(screen.getByRole('button', { name: 'Clear override' }));

  // Verify the IPC call
  expect(invoke).toHaveBeenCalledWith('clear_fact_override', {
    runFolderPath: '/test/run',
    dimension: 'empire',
    key: 'origin',
  });
});

it('saves a run note from the runs page', async () => {
  const user = userEvent.setup();
  const mockRun: PersistedRunSummary = {
    folder_path: '/test/run',
    run_folder: 'run',
    display_name: 'Test Run',
    latest_save_path: '/test/run/ironman.sav',
    latest_save_file_name: 'ironman.sav',
    latest_ingame_date: '2200.01.01',
    game_version: '4.3.0',
    parse_status: 'parsed',
    parse_error: null,
    fact_count: 1,
    updated_at: '2026-01-01',
  };

  invoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'load_runs') return [mockRun];
    if (cmd === 'load_run_facts') return [];
    if (cmd === 'load_run_notes') return null;
    return null;
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );
  expect((await screen.findAllByText('Test Run')).length).toBeGreaterThanOrEqual(1);

  // Click the run to select it
  await user.click(screen.getByRole('button', { name: /Test Run/i }));
  await screen.findByText('Run notes');

  // Type text in the note textarea
  const textarea = screen.getByPlaceholderText('Add notes for this run…');
  await user.type(textarea, 'My test note');

  // Click Save note
  await user.click(screen.getByRole('button', { name: 'Save note' }));

  // Verify the IPC call
  expect(invoke).toHaveBeenCalledWith('set_run_note', {
    runFolderPath: '/test/run',
    noteText: 'My test note',
  });
});

it('saves an achievement note from the planner', async () => {
  const user = userEvent.setup();
  const mockRun: PersistedRunSummary = {
    folder_path: '/tmp/documents/save games/run_a',
    run_folder: 'run_a',
    display_name: 'Synthetic Run',
    latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
    latest_save_file_name: 'ironman.sav',
    latest_ingame_date: '2532.01.26',
    game_version: 'Cetus v4.3.7',
    parse_status: 'parsed',
    parse_error: null,
    fact_count: 12,
    updated_at: '2026-06-03',
  };

  invoke.mockImplementation(async (cmd: string) => {
    if (cmd === 'load_runs') return [mockRun];
    if (cmd === 'load_planner_evaluations') {
      return [
        {
          achievement: {
            id: 'ach_1',
            steam_app_id: 281990,
            steam_api_name: 'ACH_ONE',
            local_key: null,
            deprecated: false,
            source: {
              name: 'First Achievement',
              description: null,
              requirement: 'Complete the thing',
              hint: null,
              group: 'Base Game',
              version_added: '1.0',
              difficulty: 'E',
            },
            curation: {
              tags: ['early'],
              conditions: [],
              warnings: [],
              planner_notes: null,
              known_limitations: [],
              rule_confidence: 'medium',
            },
          },
          status: 'Possible',
          computed_status: 'Possible',
          planned: false,
          ignored: false,
          reasons: ['No hard blocker is known.'],
          warnings: [],
          conditions: [],
        },
      ];
    }
    if (cmd === 'load_run_achievement_notes') return [];
    if (cmd === 'set_run_achievement_status') return undefined;
    return null;
  });

  render(
    <MemoryRouter>
      <Planner />
    </MemoryRouter>,
  );

  expect(await screen.findByText('First Achievement')).toBeInTheDocument();

  // Click Notes to expand the notes section
  await user.click(screen.getByRole('button', { name: /^Notes$/i }));

  // Type in the note textarea
  const textarea = screen.getByPlaceholderText('Add a note for this achievement in this run…');
  await user.type(textarea, 'My achievement note');

  // Click Save note
  await user.click(screen.getByRole('button', { name: 'Save note' }));

  // Verify the IPC call
  expect(invoke).toHaveBeenCalledWith('set_run_achievement_note', {
    runFolderPath: '/tmp/documents/save games/run_a',
    achievementId: 'ach_1',
    notes: 'My achievement note',
  });
});

it('hides Tags and Rule Confidence columns by default and supports reset', async () => {
  const user = userEvent.setup();
  invoke.mockResolvedValueOnce([]);
  invoke.mockResolvedValueOnce({
    catalog_version: '1.0',
    stellaris_version: '4.0',
    source_url: null,
    source_hash: null,
    updated_at: '2025-01-01',
    imported_at: '2025-01-02',
  });
  invoke.mockResolvedValueOnce([]);

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  await screen.findByText('Achievement Catalog');

  // Open columns panel
  await user.click(screen.getByRole('button', { name: /columns/i }));

  // Tags and Rule Confidence checkboxes should be unchecked
  const tagsCheckbox = screen.getByLabelText('Tags');
  const ruleConfCheckbox = screen.getByLabelText('Rule Confidence');
  expect(tagsCheckbox).not.toBeChecked();
  expect(ruleConfCheckbox).not.toBeChecked();

  // Reset button should be disabled initially
  const resetButton = screen.getByRole('button', { name: /reset/i });
  expect(resetButton).toBeDisabled();

  // Toggle a column, now Reset should be enabled
  await user.click(tagsCheckbox);
  expect(resetButton).toBeEnabled();

  // Click Reset
  await user.click(resetButton);
  expect(tagsCheckbox).not.toBeChecked();
});

it('shows correct sort options without Version', async () => {
  invoke.mockResolvedValueOnce([]);
  invoke.mockResolvedValueOnce({
    catalog_version: '1.0',
    stellaris_version: '4.0',
    source_url: null,
    source_hash: null,
    updated_at: '2025-01-01',
    imported_at: '2025-01-02',
  });
  invoke.mockResolvedValueOnce([]);

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  await screen.findByText('Achievement Catalog');

  const sortSelect = screen.getByLabelText('Sort achievements');
  const options = Array.from(sortSelect.querySelectorAll('option')).map((opt) => opt.textContent);

  expect(options.some((o) => o.startsWith('Name'))).toBe(true);
  expect(options.some((o) => o.startsWith('DLC'))).toBe(true);
  expect(options.some((o) => o.startsWith('Difficulty'))).toBe(true);
  expect(options.some((o) => o.startsWith('Version'))).toBe(false);
});

it('creates a force-incomplete override for Steam-baseline completed achievements', async () => {
  const user = userEvent.setup();

  // First call: loadAchievements
  invoke.mockResolvedValueOnce([
    {
      id: 'ach_steam_done',
      steam_app_id: 281990,
      steam_api_name: 'ACH_DONE',
      local_key: null,
      deprecated: false,
      completed: true, // Steam baseline complete
      source: {
        name: 'Steam Done Achievement',
        description: 'Already done',
        requirement: null,
        hint: null,
        group: 'Base Game',
        version_added: '1.0',
        difficulty: 'VE',
      },
      curation: {
        tags: [],
        conditions: [],
        warnings: [],
        planner_notes: null,
        known_limitations: [],
        rule_confidence: null,
      },
    },
  ]);
  // Second call: loadCatalogInfo
  invoke.mockResolvedValueOnce({
    catalog_version: '1.0',
    stellaris_version: '4.0',
    source_url: null,
    source_hash: null,
    updated_at: '2025-01-01',
    imported_at: '2025-01-02',
  });
  // Third call: loadCompletionOverrides (empty)
  invoke.mockResolvedValueOnce([]);

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  await screen.findByText('Steam Done Achievement');

  // Click completion toggle on the Steam-baseline item
  const toggleButton = screen.getByRole('button', { name: /set local incomplete/i });
  expect(toggleButton).toBeInTheDocument();
  await user.click(toggleButton);

  // Should have called setCompletionOverride with false (force_incomplete)
  expect(invoke).toHaveBeenCalledWith('set_completion_override', {
    achievementId: 'ach_steam_done',
    completed: false,
  });
});

it('rolls back a failed Steam-baseline force-incomplete override', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_achievements') {
      return Promise.resolve([
        {
          id: 'ach_steam_done',
          steam_app_id: 281990,
          steam_api_name: 'ACH_DONE',
          local_key: null,
          deprecated: false,
          completed: true,
          source: {
            name: 'Steam Done Achievement',
            description: 'Already done',
            requirement: null,
            hint: null,
            group: 'Base Game',
            version_added: '1.0',
            difficulty: 'VE',
          },
          curation: {
            tags: [],
            conditions: [],
            warnings: [],
            planner_notes: null,
            known_limitations: [],
            rule_confidence: null,
          },
        },
      ]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve({
        catalog_version: '1.0',
        stellaris_version: '4.0',
        source_url: null,
        source_hash: null,
        updated_at: '2025-01-01',
        imported_at: '2025-01-02',
      });
    }
    if (command === 'load_completion_overrides') return Promise.resolve([]);
    if (command === 'set_completion_override') return Promise.reject(new Error('override write failed'));
    if (command === 'get_achievement_icon') return Promise.resolve(null);
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  const toggleButton = await screen.findByRole('button', { name: /set local incomplete/i });
  expect(toggleButton).toHaveTextContent('✓');

  await user.click(toggleButton);

  expect(await screen.findByRole('alert')).toHaveTextContent('Override error: override write failed');
  expect(screen.getByRole('button', { name: /set local incomplete/i })).toHaveTextContent('✓');
});

it('renders difficulty legend without literal brackets', async () => {
  invoke.mockResolvedValueOnce([]);
  invoke.mockResolvedValueOnce({
    catalog_version: '1.0',
    stellaris_version: '4.0',
    source_url: null,
    source_hash: null,
    updated_at: '2025-01-01',
    imported_at: '2025-01-02',
  });
  invoke.mockResolvedValueOnce([]);

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  await screen.findByText('Achievement Catalog');

  const legend = screen.getByLabelText('Difficulty legend');
  expect(legend.textContent).not.toContain('[');
  expect(legend.textContent).not.toContain(']');
});

it('renders Collapse All and Expand All controls in Planner', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Collapse Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 5,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_planner_evaluations') {
      return Promise.resolve([
        {
          achievement: {
            id: 'ach_pl',
            steam_app_id: 281990,
            steam_api_name: 'ACH_PL',
            local_key: null,
            deprecated: false,
            source: {
              name: 'Plannable Achievement',
              description: null,
              requirement: 'Do the thing',
              hint: null,
              group: 'Base Game',
              version_added: '1.0',
              difficulty: 'E',
            },
            curation: {
              tags: [],
              conditions: [],
              warnings: [],
              planner_notes: null,
              known_limitations: [],
              rule_confidence: null,
            },
          },
          status: 'Possible',
          computed_status: 'Possible',
          planned: false,
          ignored: false,
          reasons: ['No blocker.'],
          warnings: [],
          conditions: [],
        },
      ]);
    }
    if (command === 'load_run_achievement_notes') return Promise.resolve([]);
    if (command === 'set_run_achievement_status') return Promise.resolve();
    return Promise.resolve([]);
  });

  render(
    <MemoryRouter>
      <Planner />
    </MemoryRouter>,
  );

  await screen.findByText('Plannable Achievement');

  // Collapse All and Expand All buttons should exist
  const collapseAll = screen.getByRole('button', { name: /collapse all/i });
  const expandAll = screen.getByRole('button', { name: /expand all/i });
  expect(collapseAll).toBeInTheDocument();
  expect(expandAll).toBeInTheDocument();

  // Click Collapse All
  await user.click(collapseAll);
  // The group should now be collapsed, so the achievement should not be visible
  expect(screen.queryByText('Plannable Achievement')).not.toBeInTheDocument();

  // Click Expand All
  await user.click(expandAll);
  expect(await screen.findByText('Plannable Achievement')).toBeInTheDocument();
});

it('shows DLC status in overview when launcher data is available', async () => {
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve({
        catalog_version: '1.1.0',
        stellaris_version: '4.3',
        source_url: null,
        source_hash: null,
        updated_at: '2026-06-03',
        imported_at: '2026-06-03',
      });
    }
    if (command === 'load_achievements') {
      return Promise.resolve([{ id: 'ach_1' }]);
    }
    if (command === 'scan_local_state') {
      return Promise.resolve({
        documents: {
          launcher: {
            dlcs: [
              { id: 'dlc-1', name: 'Utopia', registry_id: 'dlc_utopia', path: null, enabled_in_active_playset: true },
              { id: 'dlc-2', name: 'Apocalypse', registry_id: 'dlc_apocalypse', path: null, enabled_in_active_playset: false },
            ],
          },
        },
        errors: [],
      });
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  expect(await screen.findByText('1 disabled')).toBeInTheDocument();
  expect(screen.getByText(/1 enabled locally/i)).toBeInTheDocument();
});

it('explains unknown DLC status in overview when launcher data is unavailable', async () => {
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve({
        catalog_version: '1.1.0',
        stellaris_version: '4.3',
        source_url: null,
        source_hash: null,
        updated_at: '2026-06-03',
        imported_at: '2026-06-03',
      });
    }
    if (command === 'load_achievements') {
      return Promise.resolve([{ id: 'ach_1' }]);
    }
    if (command === 'scan_local_state') {
      return Promise.resolve({
        documents: {
          root: 'C:/Users/Test/OneDrive/Documents/Paradox Interactive/Stellaris',
          launcher: null,
        },
        errors: [],
      });
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  expect(await screen.findByText('Unknown')).toBeInTheDocument();
  expect(
    screen.getByText(/launcher database not found under c:\/users\/test\/onedrive\/documents\/paradox interactive\/stellaris/i),
  ).toBeInTheDocument();
});

it('shows DLC warning badges and supports DLC availability filtering on achievements', async () => {
  const user = userEvent.setup();
  invoke.mockImplementation((command: string) => {
    if (command === 'load_achievements') {
      return Promise.resolve([
        {
          id: 'ach_utopia',
          steam_app_id: 281990,
          steam_api_name: 'ACH_UTOPIA',
          local_key: null,
          deprecated: false,
          source: {
            name: 'Utopia Goal',
            description: 'Needs Utopia',
            requirement: 'Do the utopia thing',
            hint: null,
            group: 'Utopia',
            version_added: '1.5',
            difficulty: 'M',
          },
          curation: {
            tags: ['dlc'],
            conditions: [],
            warnings: [],
            planner_notes: null,
            known_limitations: [],
            rule_confidence: 'medium',
          },
        },
        {
          id: 'ach_base',
          steam_app_id: 281990,
          steam_api_name: 'ACH_BASE',
          local_key: null,
          deprecated: false,
          source: {
            name: 'Base Goal',
            description: 'Base game',
            requirement: 'Do the base thing',
            hint: null,
            group: 'Base game',
            version_added: '1.0',
            difficulty: 'E',
          },
          curation: {
            tags: [],
            conditions: [],
            warnings: [],
            planner_notes: null,
            known_limitations: [],
            rule_confidence: null,
          },
        },
      ]);
    }
    if (command === 'load_catalog_info') {
      return Promise.resolve({
        catalog_version: '1.1.0',
        stellaris_version: '4.3',
        source_url: null,
        source_hash: null,
        updated_at: '2026-06-03',
        imported_at: '2026-06-03',
      });
    }
    if (command === 'load_completion_overrides') {
      return Promise.resolve([]);
    }
    if (command === 'scan_local_state') {
      return Promise.resolve({
        documents: {
          launcher: {
            dlcs: [
              { id: 'dlc-1', name: 'Utopia', registry_id: 'dlc_utopia', path: null, enabled_in_active_playset: false },
            ],
          },
        },
        errors: [],
      });
    }
    if (command === 'get_achievement_icon') {
      return Promise.resolve(null);
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Achievements />
    </MemoryRouter>,
  );

  expect(await screen.findByText('Utopia Goal')).toBeInTheDocument();
  expect(screen.getByText('DLC not enabled')).toBeInTheDocument();

  await user.selectOptions(screen.getByLabelText('Filter by DLC availability'), 'attention');

  expect(screen.getByText('Utopia Goal')).toBeInTheDocument();
  expect(screen.queryByText('Base Goal')).not.toBeInTheDocument();
});

it('shows per-run DLC summary on the runs page', async () => {
  invoke.mockImplementation((cmd: string) => {
    if (cmd === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 2,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (cmd === 'load_run_facts') {
      return Promise.resolve([]);
    }
    if (cmd === 'load_run_notes') return Promise.resolve(null);
    if (cmd === 'scan_local_state') {
      return Promise.resolve({
        documents: {
          save_runs: [
            {
              run_folder: 'run_a',
              latest_save: { required_dlcs: ['dlc_utopia'] },
              dlc_info: {
                enabled_and_required: [],
                disabled_but_required: ['dlc_utopia'],
                unknown_status_required: [],
                all_enabled_dlcs: [],
                all_disabled_dlcs: ['dlc_utopia'],
              },
            },
          ],
        },
        errors: [],
      });
    }
    return Promise.resolve(null);
  });

  render(
    <MemoryRouter>
      <Runs />
    </MemoryRouter>,
  );

  expect((await screen.findAllByText('Synthetic Run')).length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText(/disabled but required: dlc_utopia/i)).toBeInTheDocument();
});

it('shows planner DLC warnings when a required DLC condition is unresolved', async () => {
  invoke.mockImplementation((command: string) => {
    if (command === 'load_runs') {
      return Promise.resolve([
        {
          folder_path: '/tmp/documents/save games/run_a',
          run_folder: 'run_a',
          display_name: 'Synthetic Run',
          latest_save_path: '/tmp/documents/save games/run_a/ironman.sav',
          latest_save_file_name: 'ironman.sav',
          latest_ingame_date: '2532.01.26',
          game_version: 'Cetus v4.3.7',
          parse_status: 'parsed',
          parse_error: null,
          fact_count: 12,
          updated_at: '2026-06-03',
        },
      ]);
    }
    if (command === 'load_planner_evaluations') {
      return Promise.resolve([
        {
          achievement: {
            id: 'ach_dlc',
            steam_app_id: 281990,
            steam_api_name: 'ACH_DLC',
            local_key: null,
            deprecated: false,
            source: {
              name: 'DLC Goal',
              description: null,
              requirement: 'Do the DLC thing',
              hint: null,
              group: 'Utopia',
              version_added: '1.5',
              difficulty: 'M',
            },
            curation: {
              tags: [],
              conditions: [],
              warnings: [],
              planner_notes: null,
              known_limitations: [],
              rule_confidence: 'medium',
            },
          },
          status: 'Unknown',
          computed_status: 'Unknown',
          planned: false,
          ignored: false,
          reasons: ['Needs DLC confirmation.'],
          warnings: [],
          conditions: [
            {
              dimension: 'required_dlc',
              operator: 'contains',
              condition_value: 'utopia',
              fact_value: null,
              passed: null,
              severity: 'soft',
              timing: 'setup',
              mutability: 'normal_change',
              reason: 'Requires Utopia DLC (not currently confirmed).',
            },
          ],
        },
      ]);
    }
    if (command === 'load_run_achievement_notes') return Promise.resolve([]);
    if (command === 'set_run_achievement_status') return Promise.resolve();
    return Promise.resolve([]);
  });

  render(
    <MemoryRouter>
      <Planner />
    </MemoryRouter>,
  );

  expect(await screen.findByText('DLC Goal')).toBeInTheDocument();
  // Compact DLC blocker summary shown instead of verbose raw reason
  expect(screen.getByText(/Missing: utopia/i)).toBeInTheDocument();
  expect(screen.getByText(/DLC attention/i)).toBeInTheDocument();
});
