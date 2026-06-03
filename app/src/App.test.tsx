import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, expect, it, vi } from 'vitest';

const invoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({
  invoke,
}));

import { App } from './App';
import { Overview } from './pages/Overview';
import { Achievements } from './pages/Achievements';
import { Runs } from './pages/Runs';

beforeEach(() => {
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
    if (command === 'load_run_facts') {
      return Promise.resolve([]);
    }
    if (command === 'rescan_saves') {
      return Promise.resolve([]);
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
  expect(screen.getByRole('link', { name: /runs/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /settings/i })).toBeInTheDocument();
  expect(await screen.findByRole('button', { name: /rescan saves/i })).toBeInTheDocument();
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

it('renders achievements page with mocked data', async () => {
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

  expect(await screen.findByText('First Achievement')).toBeInTheDocument();
  expect(screen.getByRole('cell', { name: 'Base Game' })).toBeInTheDocument();
});

it('invokes the scan command with an empty payload', async () => {
  invoke.mockResolvedValueOnce({ errors: [] });

  const { scanLocalState } = await import('./tauri');
  await scanLocalState();

  expect(invoke).toHaveBeenCalledWith('scan_local_state', {});
});

it('loads persisted runs and facts on the runs page', async () => {
  const user = userEvent.setup();
  invoke.mockResolvedValueOnce([
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
  invoke.mockResolvedValueOnce([
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
  invoke.mockResolvedValueOnce([]);
  invoke.mockResolvedValueOnce([
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
  invoke.mockResolvedValueOnce([]);

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
