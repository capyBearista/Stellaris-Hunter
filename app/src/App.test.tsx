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

beforeEach(() => {
  invoke.mockReset();
});

it('renders the app shell with navigation links', () => {
  render(<App />);

  expect(screen.getAllByText('Stellaris Hunter').length).toBeGreaterThanOrEqual(1);
  expect(screen.getByRole('link', { name: /overview/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /achievements/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /runs/i })).toBeInTheDocument();
  expect(screen.getByRole('link', { name: /settings/i })).toBeInTheDocument();
});

it('renders overview page with heading and scan button', () => {
  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  expect(screen.getByRole('heading', { name: /overview/i })).toBeInTheDocument();
  expect(screen.getByRole('button', { name: /scan local files/i })).toBeInTheDocument();
});

it('invokes the scan command and shows the returned report', async () => {
  const user = userEvent.setup();
  let resolveReport: (value: unknown) => void = () => undefined;

  invoke.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        resolveReport = resolve;
      }),
  );

  render(
    <MemoryRouter>
      <Overview />
    </MemoryRouter>,
  );

  await user.click(screen.getByRole('button', { name: /scan local files/i }));

  expect(invoke).toHaveBeenCalledWith('scan_local_state', {});
  expect(screen.getByRole('button', { name: /scanning/i })).toBeDisabled();

  resolveReport({
    errors: [],
    install: { version: '4.0.0', root: '/tmp/install' },
    documents: {
      root: '/tmp/documents',
      save_runs: [
        {
          run_folder: 'run_a',
          latest_save: {
            name: 'Synthetic Run',
            date: '2532.01.26',
          },
        },
      ],
    },
  });

  expect(await screen.findByText('4.0.0')).toBeInTheDocument();
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
