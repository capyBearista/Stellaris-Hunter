import '@testing-library/jest-dom/vitest';

// Simulate Tauri runtime bridge so the IPC wrappers in tauri.ts use the
// invoke() path (which is mocked by App.test.tsx) instead of fetch().
Object.defineProperty(window, '__TAURI_INTERNALS__', {
  value: { invoke: true },
  writable: true,
  configurable: true,
});
