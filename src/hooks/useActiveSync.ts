/**
 * React hook for managing active sync state
 * Provides control over the bidirectional sync engine
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/**
 * Latency statistics from the sync engine
 */
export interface LatencyStats {
  avg_ms: number;
  min_ms: number;
  max_ms: number;
  sample_count: number;
}

/**
 * Sync status information
 */
export interface SyncStatus {
  /** Whether sync engine is initialized */
  initialized: boolean;
  /** Whether sync is actively running */
  active: boolean;
  /** Number of active mappings */
  mappingCount: number;
  /** Current latency statistics */
  latencyStats: LatencyStats | null;
}

export interface UseActiveSyncReturn {
  /** Current sync status */
  status: SyncStatus;
  /** Whether data is loading */
  loading: boolean;
  /** Error message if any */
  error: string | null;
  /** Initialize the sync engine */
  initializeSync: () => Promise<boolean>;
  /** Start syncing (auto-initializes if needed) */
  startSync: () => Promise<boolean>;
  /** Stop syncing */
  stopSync: () => Promise<boolean>;
  /** Refresh sync status */
  refreshStatus: () => Promise<void>;
  /** Clear latency statistics */
  clearLatencyStats: () => Promise<void>;
}

/**
 * Hook for managing active bidirectional sync
 * 
 * @param autoInit - Whether to auto-initialize sync engine on mount (default: true)
 * @param pollInterval - How often to poll for status updates (ms), default 1000ms
 * @returns Sync state and control functions
 * 
 * @example
 * ```tsx
 * const { status, startSync, stopSync } = useActiveSync();
 * 
 * // Start syncing
 * await startSync();
 * 
 * // Check latency
 * console.log(status.latencyStats?.avg_ms);
 * ```
 */
export function useActiveSync(
  autoInit: boolean = true,
  pollInterval: number = 1000
): UseActiveSyncReturn {
  const [status, setStatus] = useState<SyncStatus>({
    initialized: false,
    active: false,
    mappingCount: 0,
    latencyStats: null,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetches current sync status from the backend
   */
  const refreshStatus = useCallback(async () => {
    try {
      setError(null);

      // Get mapping count
      const mappings = await invoke<unknown[]>('get_parameter_mappings');
      const mappingCount = mappings.length;

      // Get latency stats
      const latencyStats = await invoke<LatencyStats | null>('get_latency_stats');

      setStatus(prev => ({
        ...prev,
        mappingCount,
        latencyStats,
        active: mappingCount > 0 && prev.initialized,
      }));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Initializes the sync engine
   */
  const initializeSync = useCallback(async (): Promise<boolean> => {
    try {
      setError(null);
      await invoke('init_sync_engine');
      setStatus(prev => ({ ...prev, initialized: true }));
      await refreshStatus();
      return true;
    } catch (err) {
      // If already initialized, that's okay
      if (String(err).includes('already initialized')) {
        setStatus(prev => ({ ...prev, initialized: true }));
        await refreshStatus();
        return true;
      }
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, [refreshStatus]);

  /**
   * Starts syncing (auto-initializes if needed)
   */
  const startSync = useCallback(async (): Promise<boolean> => {
    try {
      setError(null);

      // Initialize if not already done
      if (!status.initialized) {
        const initialized = await initializeSync();
        if (!initialized) {
          return false;
        }
      }

      // Start the sync integration (wires up MIDI callbacks)
      await invoke('start_sync_integration');

      setStatus(prev => ({ ...prev, active: true }));
      await refreshStatus();
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, [status.initialized, initializeSync, refreshStatus]);

  /**
   * Stops syncing
   */
  const stopSync = useCallback(async (): Promise<boolean> => {
    try {
      setError(null);
      await invoke('stop_sync_integration');
      setStatus(prev => ({ ...prev, active: false }));
      return true;
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      return false;
    }
  }, []);

  /**
   * Clears latency statistics
   */
  const clearLatencyStats = useCallback(async () => {
    try {
      setError(null);
      await invoke('clear_latency_stats');
      await refreshStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [refreshStatus]);

  /**
   * Auto-initialize on mount if requested
   */
  useEffect(() => {
    if (autoInit) {
      initializeSync();
    } else {
      refreshStatus();
    }
  }, [autoInit, initializeSync, refreshStatus]);

  /**
   * Poll for status updates when active
   */
  useEffect(() => {
    if (!status.active) {
      return;
    }

    const interval = setInterval(refreshStatus, pollInterval);
    return () => clearInterval(interval);
  }, [status.active, pollInterval, refreshStatus]);

  /**
   * Listen for sync events from backend
   */
  useEffect(() => {
    const unlisten = listen('sync:parameter-synced', () => {
      refreshStatus();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [refreshStatus]);

  return {
    status,
    loading,
    error,
    initializeSync,
    startSync,
    stopSync,
    refreshStatus,
    clearLatencyStats,
  };
}
