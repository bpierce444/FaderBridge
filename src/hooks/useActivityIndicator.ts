/**
 * Hook for tracking activity indicators with auto-timeout
 * Used to show MIDI and UCNet activity lights in the status bar
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export interface UseActivityIndicatorOptions {
  /** Event names to listen for */
  eventNames: string[];
  /** Timeout in milliseconds before activity indicator turns off (default: 500ms) */
  timeout?: number;
}

export interface UseActivityIndicatorResult {
  /** Whether activity is currently detected */
  isActive: boolean;
  /** Manually trigger activity (useful for testing) */
  triggerActivity: () => void;
  /** Reset activity state */
  reset: () => void;
}

/**
 * Hook to track activity from Tauri events with auto-timeout.
 * Listens for specified events and shows activity for a brief period.
 * 
 * @example
 * ```tsx
 * const midiActivity = useActivityIndicator({
 *   eventNames: ['midi:message-received', 'parameter-update'],
 *   timeout: 500,
 * });
 * 
 * return <ActivityLight active={midiActivity.isActive} />;
 * ```
 */
export function useActivityIndicator({
  eventNames,
  timeout = 500,
}: UseActivityIndicatorOptions): UseActivityIndicatorResult {
  const [isActive, setIsActive] = useState(false);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const timeoutValueRef = useRef(timeout);

  // Keep timeout value ref in sync
  timeoutValueRef.current = timeout;

  /**
   * Triggers activity and resets the timeout
   */
  const triggerActivity = useCallback(() => {
    setIsActive(true);

    // Clear existing timeout
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    // Set new timeout to deactivate
    timeoutRef.current = setTimeout(() => {
      setIsActive(false);
      timeoutRef.current = null;
    }, timeoutValueRef.current);
  }, []);

  /**
   * Resets activity state immediately
   */
  const reset = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setIsActive(false);
  }, []);

  /**
   * Listen for events from the backend
   */
  useEffect(() => {
    const unlistenFns: Promise<UnlistenFn>[] = [];

    for (const eventName of eventNames) {
      const unlisten = listen(eventName, () => {
        triggerActivity();
      });
      unlistenFns.push(unlisten);
    }

    return () => {
      // Cleanup all listeners
      for (const unlisten of unlistenFns) {
        unlisten.then((fn) => fn());
      }
      // Clear timeout on unmount
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [eventNames, triggerActivity]);

  return {
    isActive,
    triggerActivity,
    reset,
  };
}
