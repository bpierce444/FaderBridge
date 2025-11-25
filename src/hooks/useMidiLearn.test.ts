/**
 * Tests for useMidiLearn hook
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useMidiLearn } from './useMidiLearn';
import { LearnState } from '../types/learn';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe('useMidiLearn', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should initialize with idle state', async () => {
    const idleState: LearnState = { type: 'idle' };
    mockInvoke.mockResolvedValue(idleState);

    const { result } = renderHook(() => useMidiLearn());

    await waitFor(() => {
      expect(result.current.learnState).toEqual(idleState);
    });
    
    expect(result.current.isLearning).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it('should start learn mode', async () => {
    const idleState: LearnState = { type: 'idle' };
    const listeningState: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 0,
    };

    mockInvoke
      .mockResolvedValueOnce(idleState) // Initial state
      .mockResolvedValueOnce(true) // start_midi_learn
      .mockResolvedValueOnce(listeningState); // get_midi_learn_state after start

    const { result } = renderHook(() => useMidiLearn());

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('idle');
    });

    let startResult: boolean = false;
    await act(async () => {
      startResult = await result.current.startLearn('device-1', 1, 'volume');
    });

    expect(startResult).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith('start_midi_learn', {
      deviceId: 'device-1',
      channel: 1,
      parameterType: 'volume',
    });

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('listening');
      expect(result.current.isLearning).toBe(true);
    });
  });

  it('should cancel learn mode', async () => {
    const listeningState: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 1000,
    };
    const idleState: LearnState = { type: 'idle' };

    mockInvoke
      .mockResolvedValueOnce(listeningState) // Initial state
      .mockResolvedValueOnce(undefined) // cancel_midi_learn
      .mockResolvedValueOnce(idleState); // get_midi_learn_state after cancel

    const { result } = renderHook(() => useMidiLearn());

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('listening');
    });

    await act(async () => {
      await result.current.cancelLearn();
    });

    expect(mockInvoke).toHaveBeenCalledWith('cancel_midi_learn');

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('idle');
      expect(result.current.isLearning).toBe(false);
    });
  });

  it('should poll for state updates when learning', async () => {
    const listeningState1: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 1000,
    };
    const listeningState2: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 2000,
    };

    mockInvoke
      .mockResolvedValueOnce(listeningState1) // Initial state
      .mockResolvedValueOnce(listeningState2) // First poll
      .mockResolvedValue(listeningState2); // Subsequent polls

    // Use a very short poll interval for testing
    const { result } = renderHook(() => useMidiLearn(50));

    // Wait for initial state
    await waitFor(() => {
      expect(result.current.learnState.type).toBe('listening');
    });

    // Wait for polling to occur
    await waitFor(() => {
      expect(mockInvoke.mock.calls.length).toBeGreaterThanOrEqual(2);
    });
  });

  it('should handle ESC key to cancel learn mode', async () => {
    const listeningState: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 1000,
    };
    const idleState: LearnState = { type: 'idle' };

    mockInvoke
      .mockResolvedValueOnce(listeningState) // Initial state
      .mockResolvedValueOnce(undefined) // cancel_midi_learn
      .mockResolvedValue(idleState); // get_midi_learn_state after cancel

    const { result } = renderHook(() => useMidiLearn(10000)); // Long poll to avoid interference

    // Wait for listening state
    await waitFor(() => {
      expect(result.current.learnState.type).toBe('listening');
    });

    // Simulate ESC key press
    await act(async () => {
      const event = new KeyboardEvent('keydown', { key: 'Escape' });
      window.dispatchEvent(event);
    });

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('cancel_midi_learn');
    });
  });

  it('should handle errors when starting learn mode', async () => {
    const idleState: LearnState = { type: 'idle' };
    const error = new Error('Failed to start learn mode');

    mockInvoke
      .mockResolvedValueOnce(idleState) // Initial state
      .mockRejectedValueOnce(error); // start_midi_learn error

    const { result } = renderHook(() => useMidiLearn(10000)); // Long poll to avoid interference

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('idle');
    });

    let startResult: boolean = false;
    await act(async () => {
      startResult = await result.current.startLearn('device-1', 1, 'volume');
    });

    expect(startResult).toBe(false);
    await waitFor(() => {
      expect(result.current.error).toBe('Failed to start learn mode');
    });
  });

  it('should handle errors when cancelling learn mode', async () => {
    const listeningState: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 1000,
    };
    const error = new Error('Failed to cancel learn mode');

    mockInvoke
      .mockResolvedValueOnce(listeningState) // Initial state
      .mockRejectedValueOnce(error); // cancel_midi_learn error

    const { result } = renderHook(() => useMidiLearn(10000)); // Long poll to avoid interference

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('listening');
    });

    await act(async () => {
      await result.current.cancelLearn();
    });

    await waitFor(() => {
      expect(result.current.error).toBe('Failed to cancel learn mode');
    });
  });

  it('should refresh state manually', async () => {
    const idleState: LearnState = { type: 'idle' };
    const listeningState: LearnState = {
      type: 'listening',
      device_id: 'device-1',
      channel: 1,
      parameter_type: 'volume',
      elapsed_ms: 1000,
    };

    mockInvoke
      .mockResolvedValueOnce(idleState) // Initial state
      .mockResolvedValueOnce(listeningState); // Manual refresh

    const { result } = renderHook(() => useMidiLearn(10000)); // Long poll to avoid interference

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('idle');
    });

    await act(async () => {
      await result.current.refreshState();
    });

    await waitFor(() => {
      expect(result.current.learnState.type).toBe('listening');
    });
  });

  it('should not poll when not in learn mode', async () => {
    const idleState: LearnState = { type: 'idle' };

    mockInvoke.mockResolvedValue(idleState);

    const { result } = renderHook(() => useMidiLearn(50)); // Short poll interval

    // Wait for initial state fetch
    await waitFor(() => {
      expect(result.current.learnState.type).toBe('idle');
    });
    
    expect(mockInvoke).toHaveBeenCalledWith('get_midi_learn_state');

    const callCount = mockInvoke.mock.calls.length;

    // Wait a bit - should not poll when idle
    await new Promise(resolve => setTimeout(resolve, 100));

    // Should not have made additional calls (no polling when idle)
    expect(mockInvoke.mock.calls.length).toBe(callCount);
  });
});
