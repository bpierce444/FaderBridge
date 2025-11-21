import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAutoSave } from './useAutoSave';

// Mock the Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(() => Promise.resolve()),
}));

import { invoke } from '@tauri-apps/api/core';

describe('useAutoSave', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('does not save if projectId is null', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: null,
        interval: 1000,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      vi.advanceTimersByTime(1000);
      await Promise.resolve();
    });

    expect(invoke).not.toHaveBeenCalled();
  });

  it('does not save if not marked dirty', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 1000,
      })
    );

    await act(async () => {
      vi.advanceTimersByTime(1000);
      await Promise.resolve();
    });

    expect(invoke).not.toHaveBeenCalled();
  });

  it('saves after interval when marked dirty', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 1000,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      vi.advanceTimersByTime(1000);
      await Promise.resolve();
    });

    expect(invoke).toHaveBeenCalledWith('update_project', {
      req: {
        id: 1,
        name: undefined,
        description: undefined,
      },
    });
  });

  it('does not save again if not marked dirty after save', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 1000,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      vi.advanceTimersByTime(1000);
      await Promise.resolve();
    });

    expect(invoke).toHaveBeenCalledTimes(1);

    vi.clearAllMocks();

    await act(async () => {
      vi.advanceTimersByTime(1000);
      await Promise.resolve();
    });

    expect(invoke).not.toHaveBeenCalled();
  });

  it('saves immediately when saveNow is called', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 10000,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      await result.current.saveNow();
    });

    expect(invoke).toHaveBeenCalledWith('update_project', {
      req: {
        id: 1,
        name: undefined,
        description: undefined,
      },
    });
  });

  it('calls onSuccess callback when save succeeds', async () => {
    const onSuccess = vi.fn();
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 1000,
        onSuccess,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      await result.current.saveNow();
    });

    expect(onSuccess).toHaveBeenCalledTimes(1);
  });

  it('calls onError callback when save fails', async () => {
    const onError = vi.fn();
    const error = new Error('Save failed');
    vi.mocked(invoke).mockRejectedValueOnce(error);

    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 1000,
        onError,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      await result.current.saveNow();
    });

    expect(onError).toHaveBeenCalledWith(error);
  });

  it('does not save when disabled', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 1000,
        enabled: false,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      vi.advanceTimersByTime(1000);
      await Promise.resolve();
    });

    expect(invoke).not.toHaveBeenCalled();
  });

  it('respects custom interval', async () => {
    const { result } = renderHook(() =>
      useAutoSave({
        projectId: 1,
        interval: 5000,
      })
    );

    act(() => {
      result.current.markDirty();
    });

    await act(async () => {
      vi.advanceTimersByTime(3000);
      await Promise.resolve();
    });

    expect(invoke).not.toHaveBeenCalled();

    await act(async () => {
      vi.advanceTimersByTime(2000);
      await Promise.resolve();
    });

    expect(invoke).toHaveBeenCalled();
  });
});
