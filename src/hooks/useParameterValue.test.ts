import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useParameterValue } from './useParameterValue';

// Mock the Tauri API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe('useParameterValue', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('initializes with default value', () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.75,
      })
    );

    expect(result.current.value).toBe(0.75);
    expect(result.current.isActive).toBe(false);
    expect(result.current.lastUpdate).toBe(null);
  });

  it('updates value when setValue is called', () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
      })
    );

    act(() => {
      result.current.setValue(0.75);
    });

    expect(result.current.value).toBe(0.75);
    expect(result.current.isActive).toBe(true);
    expect(result.current.lastUpdate).not.toBe(null);
  });

  it('clamps value between 0 and 1', () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
      })
    );

    act(() => {
      result.current.setValue(1.5);
    });
    expect(result.current.value).toBe(1);

    act(() => {
      result.current.setValue(-0.5);
    });
    expect(result.current.value).toBe(0);
  });

  it('calls onChange callback when value changes', () => {
    const onChange = vi.fn();
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
        onChange,
      })
    );

    act(() => {
      result.current.setValue(0.75);
    });

    expect(onChange).toHaveBeenCalledWith(0.75);
  });

  it('sets isActive to true when value changes', () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
      })
    );

    expect(result.current.isActive).toBe(false);

    act(() => {
      result.current.setValue(0.75);
    });

    expect(result.current.isActive).toBe(true);
  });

  it('deactivates after timeout', async () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
        activityTimeout: 500,
      })
    );

    act(() => {
      result.current.setValue(0.75);
    });

    expect(result.current.isActive).toBe(true);

    await act(async () => {
      vi.advanceTimersByTime(500);
      await Promise.resolve();
    });

    expect(result.current.isActive).toBe(false);
  });

  it('respects custom activity timeout', async () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
        activityTimeout: 1000,
      })
    );

    act(() => {
      result.current.setValue(0.75);
    });

    expect(result.current.isActive).toBe(true);

    await act(async () => {
      vi.advanceTimersByTime(500);
      await Promise.resolve();
    });

    expect(result.current.isActive).toBe(true);

    await act(async () => {
      vi.advanceTimersByTime(500);
      await Promise.resolve();
    });

    expect(result.current.isActive).toBe(false);
  });

  it('resets timeout on subsequent value changes', async () => {
    const { result } = renderHook(() =>
      useParameterValue({
        parameter: 'line/ch1/vol',
        initialValue: 0.5,
        activityTimeout: 500,
      })
    );

    act(() => {
      result.current.setValue(0.6);
    });

    await act(async () => {
      vi.advanceTimersByTime(400);
      await Promise.resolve();
    });

    expect(result.current.isActive).toBe(true);

    // Change value again before timeout
    act(() => {
      result.current.setValue(0.7);
    });

    // Advance by 400ms (total 800ms from first change, but only 400ms from second)
    await act(async () => {
      vi.advanceTimersByTime(400);
      await Promise.resolve();
    });

    expect(result.current.isActive).toBe(true);

    // Advance by another 100ms to complete the 500ms from second change
    await act(async () => {
      vi.advanceTimersByTime(100);
      await Promise.resolve();
    });

    expect(result.current.isActive).toBe(false);
  });
});
