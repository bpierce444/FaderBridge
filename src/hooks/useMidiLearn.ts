/**
 * React hook for MIDI Learn functionality
 * Provides state management and commands for MIDI Learn mode
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LearnState, isLearning } from '../types/learn';
import { UcNetParameterType } from '../types/mapping';

export interface UseMidiLearnReturn {
  /** Current learn state */
  learnState: LearnState;
  /** Whether currently in learn mode */
  isLearning: boolean;
  /** Start MIDI Learn for a parameter */
  startLearn: (deviceId: string, channel: number, parameterType: UcNetParameterType) => Promise<boolean>;
  /** Cancel MIDI Learn mode */
  cancelLearn: () => Promise<void>;
  /** Refresh the learn state */
  refreshState: () => Promise<void>;
  /** Error message if any */
  error: string | null;
}

/**
 * Hook for managing MIDI Learn functionality
 * 
 * @param pollInterval - How often to poll for state updates (ms), default 500ms
 * @returns MIDI Learn state and control functions
 * 
 * @example
 * ```tsx
 * const { learnState, isLearning, startLearn, cancelLearn } = useMidiLearn();
 * 
 * // Start learning for volume on channel 1
 * await startLearn('device-1', 1, 'volume');
 * 
 * // Cancel learning
 * await cancelLearn();
 * ```
 */
export function useMidiLearn(pollInterval: number = 500): UseMidiLearnReturn {
  const [learnState, setLearnState] = useState<LearnState>({ type: 'idle' });
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetches the current learn state from the backend
   */
  const refreshState = useCallback(async () => {
    try {
      const state = await invoke<LearnState>('get_midi_learn_state');
      setLearnState(state);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  /**
   * Starts MIDI Learn mode for a specific parameter
   */
  const startLearn = useCallback(
    async (deviceId: string, channel: number, parameterType: UcNetParameterType): Promise<boolean> => {
      try {
        const result = await invoke<boolean>('start_midi_learn', {
          deviceId,
          channel,
          parameterType,
        });
        
        if (result) {
          await refreshState();
        }
        
        setError(null);
        return result;
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        return false;
      }
    },
    [refreshState]
  );

  /**
   * Cancels MIDI Learn mode
   */
  const cancelLearn = useCallback(async () => {
    try {
      await invoke('cancel_midi_learn');
      await refreshState();
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [refreshState]);

  /**
   * Poll for state updates when in learn mode
   */
  useEffect(() => {
    if (!isLearning(learnState)) {
      return;
    }

    const interval = setInterval(refreshState, pollInterval);
    return () => clearInterval(interval);
  }, [learnState, pollInterval, refreshState]);

  /**
   * Listen for ESC key to cancel learn mode
   */
  useEffect(() => {
    if (!isLearning(learnState)) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        cancelLearn();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [learnState, cancelLearn]);

  /**
   * Initial state fetch
   */
  useEffect(() => {
    refreshState();
  }, [refreshState]);

  return {
    learnState,
    isLearning: isLearning(learnState),
    startLearn,
    cancelLearn,
    refreshState,
    error,
  };
}
