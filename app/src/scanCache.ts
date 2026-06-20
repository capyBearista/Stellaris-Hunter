import { scanLocalState, type ScanReport } from './tauri';

// ---------------------------------------------------------------------------
// Module-level scan cache
// ---------------------------------------------------------------------------

let cachedReport: ScanReport | null = null;
let inflightPromise: Promise<ScanReport> | null = null;
let inflightGeneration: number | null = null;
let cacheValid = false;
let cacheGeneration = 0;

/**
 * Returns the cached scan report, or `null` when no valid cache exists.
 * After explicit invalidation this returns `null` until the next fresh scan
 * completes.
 */
export function getCachedScanReport(): ScanReport | null {
  return cacheValid ? cachedReport : null;
}

/**
 * Invalidates the cache so the next call to `scanLocalStateCached()` performs
 * a fresh scan.
 */
export function invalidateScanCache(): void {
  cacheValid = false;
  cacheGeneration += 1;
}

/**
 * Returns a cached scan report when the cache is valid, deduplicates
 * in-flight scan requests, and only performs a fresh scan when the cache
 * is invalid or missing.
 *
 * - **Valid cache** → resolves immediately with the cached report.
 * - **In-flight request** → joins the existing promise (no duplicate scan).
 * - **No cache or invalidated** → calls `scanLocalState()` and caches the
 *   result.
 */
export async function scanLocalStateCached(options?: { force?: boolean }): Promise<ScanReport> {
  if (options?.force) {
    invalidateScanCache();
  }

  if (cacheValid && cachedReport !== null) {
    return cachedReport;
  }

  if (inflightPromise && inflightGeneration === cacheGeneration) {
    return inflightPromise;
  }

  const requestGeneration = cacheGeneration;

  let requestPromise: Promise<ScanReport>;
  requestPromise = scanLocalState()
    .then((report) => {
      if (requestGeneration === cacheGeneration) {
        cachedReport = report;
        cacheValid = true;
      }
      return report;
    })
    .finally(() => {
      if (inflightPromise === requestPromise) {
        inflightPromise = null;
        inflightGeneration = null;
      }
    });

  inflightPromise = requestPromise;
  inflightGeneration = requestGeneration;

  return requestPromise;
}

/**
 * Resets the cache to its initial empty state.  Intended for test isolation;
 * call in `beforeEach` when using the cache module.
 */
export function resetScanCache(): void {
  cachedReport = null;
  inflightPromise = null;
  inflightGeneration = null;
  cacheValid = false;
  cacheGeneration = 0;
}
