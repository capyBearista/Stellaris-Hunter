import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, expect, it, vi } from 'vitest';

const invoke = vi.hoisted(() => vi.fn());

vi.mock('@tauri-apps/api/core', () => ({
  invoke,
}));

import { App } from './App';

beforeEach(() => {
  invoke.mockReset();
});

it('renders the desktop shell', () => {
  render(<App />);

  expect(screen.getByRole('heading', { name: /stellaris hunter/i })).toBeInTheDocument();
  expect(screen.getByRole('heading', { name: /overview/i })).toBeInTheDocument();
  expect(screen.getByRole('heading', { name: /runs\/saves/i })).toBeInTheDocument();
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

  render(<App />);
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

  expect(await screen.findByText('run_a')).toBeInTheDocument();
  expect(screen.getByText('Synthetic Run', { selector: 'span' })).toBeInTheDocument();
});

it('invokes the scan command with an empty payload', async () => {
  invoke.mockResolvedValueOnce({ errors: [] });

  const { scanLocalState } = await import('./tauri');
  await scanLocalState();

  expect(invoke).toHaveBeenCalledWith('scan_local_state', {});
});
