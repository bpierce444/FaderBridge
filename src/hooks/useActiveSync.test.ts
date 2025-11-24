/**
 * Tests for useActiveSync hook
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useActiveSync } from './useActiveSync';
import * as tauriCore from '@tauri-apps/api/core';
import * as tauriEvent from '@tauri-apps/api/event';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core');
vi.mock('@tauri-apps/api/event');

describe('useActiveSync', () => {
  const mockInvoke = vi.mocked(tauriCore.invoke);
  const mockListen = vi.mocked(tauriEvent.listen);

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Default mock implementations
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_parameter_mappings') {
        return [];
      }
      if (cmd === 'get_latency_stats') {
        return null;
      }
      if (cmd === 'init_sync_engine') {
        return undefined;
      }
      return undefined;
    });

    mockListen.mockResolvedValue(() => {});
  });

  it('initializes with default state', async () => {
    const { result } = renderHook(() => useActiveSync(false));

    expect(result.current.status.initialized).toBe(false);
    expect(result.current.status.active).toBe(false);
    expect(result.current.status.mappingCount).toBe(0);
    expect(result.current.status.latencyStats).toBeNull();
  });

  it('auto-initializes when autoInit is true', async () => {
    renderHook(() => useActiveSync(true));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('init_sync_engine');
    });
  });

  it('does not auto-initialize when autoInit is false', async () => {
    renderHook(() => useActiveSync(false));

    await waitFor(() => {
      expect(mockInvoke).not.toHaveBeenCalledWith('init_sync_engine');
    });
  });

  it('initializes sync engine successfully', async () => {
    const { result } = renderHook(() => useActiveSync(false));

    const success = await result.current.initializeSync();

    expect(success).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('init_sync_engine');
    
    await waitFor(() => {
      expect(result.current.status.initialized).toBe(true);
    });
  });

  it('handles already initialized error gracefully', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'init_sync_engine') {
        throw new Error('Sync engine already initialized');
      }
      if (cmd === 'get_parameter_mappings') {
        return [];
      }
      if (cmd === 'get_latency_stats') {
        return null;
      }
      return undefined;
    });

    const { result } = renderHook(() => useActiveSync(false));

    const success = await result.current.initializeSync();

    expect(success).toBe(true);
    await waitFor(() => {
      expect(result.current.status.initialized).toBe(true);
    });
  });

  it('starts sync successfully', async () => {
    const { result } = renderHook(() => useActiveSync(false));

    // Initialize first
    await result.current.initializeSync();

    const success = await result.current.startSync();

    expect(success).toBe(true);
    await waitFor(() => {
      expect(result.current.status.active).toBe(true);
    });
  });

  it('auto-initializes when starting sync if not initialized', async () => {
    const { result } = renderHook(() => useActiveSync(false));

    const success = await result.current.startSync();

    expect(success).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('init_sync_engine');
    
    await waitFor(() => {
      expect(result.current.status.initialized).toBe(true);
      expect(result.current.status.active).toBe(true);
    });
  });

  it('stops sync successfully', async () => {
    const { result } = renderHook(() => useActiveSync(false));

    await result.current.startSync();
    const success = await result.current.stopSync();

    expect(success).toBe(true);
    await waitFor(() => {
      expect(result.current.status.active).toBe(false);
    });
  });

  it('refreshes status with mapping count', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_parameter_mappings') {
        return [{ id: 1 }, { id: 2 }, { id: 3 }];
      }
      if (cmd === 'get_latency_stats') {
        return null;
      }
      return undefined;
    });

    const { result } = renderHook(() => useActiveSync(false));

    await result.current.refreshStatus();

    await waitFor(() => {
      expect(result.current.status.mappingCount).toBe(3);
    });
  });

  it('refreshes status with latency stats', async () => {
    const mockStats = {
      avg_ms: 5.5,
      min_ms: 2.0,
      max_ms: 12.0,
      sample_count: 100,
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_parameter_mappings') {
        return [];
      }
      if (cmd === 'get_latency_stats') {
        return mockStats;
      }
      return undefined;
    });

    const { result } = renderHook(() => useActiveSync(false));

    await result.current.refreshStatus();

    await waitFor(() => {
      expect(result.current.status.latencyStats).toEqual(mockStats);
    });
  });

  it('clears latency stats', async () => {
    const { result } = renderHook(() => useActiveSync(false));

    await result.current.clearLatencyStats();

    expect(mockInvoke).toHaveBeenCalledWith('clear_latency_stats');
  });

  it('handles errors gracefully', async () => {
    mockInvoke.mockRejectedValue(new Error('Test error'));

    const { result } = renderHook(() => useActiveSync(false));

    const success = await result.current.initializeSync();

    expect(success).toBe(false);
    await waitFor(() => {
      expect(result.current.error).toBe('Test error');
    });
  });

  it('listens for sync events', async () => {
    renderHook(() => useActiveSync(false));

    await waitFor(() => {
      expect(mockListen).toHaveBeenCalledWith('sync:parameter-synced', expect.any(Function));
    });
  });

  it('sets active to true when initialized and has mappings', async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'get_parameter_mappings') {
        return [{ id: 1 }];
      }
      if (cmd === 'get_latency_stats') {
        return null;
      }
      if (cmd === 'init_sync_engine') {
        return undefined;
      }
      return undefined;
    });

    const { result } = renderHook(() => useActiveSync(false));

    await result.current.initializeSync();

    await waitFor(() => {
      expect(result.current.status.active).toBe(true);
      expect(result.current.status.mappingCount).toBe(1);
    });
  });
});
