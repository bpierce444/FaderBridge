import { useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { UpdateProjectRequest } from '../types/projects';

export interface UseAutoSaveOptions {
  /** Project ID to auto-save */
  projectId: number | null;
  /** Auto-save interval in milliseconds (default: 30000 = 30 seconds) */
  interval?: number;
  /** Whether auto-save is enabled (default: true) */
  enabled?: boolean;
  /** Callback when auto-save succeeds */
  onSuccess?: () => void;
  /** Callback when auto-save fails */
  onError?: (error: Error) => void;
}

export interface UseAutoSaveReturn {
  /** Trigger an immediate save */
  saveNow: () => Promise<void>;
  /** Mark that changes have been made (resets the timer) */
  markDirty: () => void;
  /** Whether a save is currently in progress */
  isSaving: boolean;
  /** Last save timestamp */
  lastSaved: Date | null;
}

/**
 * Hook to automatically save project changes at regular intervals.
 * Only saves if changes have been detected since the last save.
 */
export function useAutoSave({
  projectId,
  interval = 30000,
  enabled = true,
  onSuccess,
  onError,
}: UseAutoSaveOptions): UseAutoSaveReturn {
  const isDirtyRef = useRef(false);
  const isSavingRef = useRef(false);
  const lastSavedRef = useRef<Date | null>(null);
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  const saveNow = useCallback(async () => {
    if (!projectId || isSavingRef.current || !isDirtyRef.current) {
      return;
    }

    isSavingRef.current = true;

    try {
      // Update the project's last_modified timestamp
      const request: UpdateProjectRequest = {
        id: projectId,
        name: undefined,
        description: undefined,
      };

      await invoke('update_project', { req: request });
      
      isDirtyRef.current = false;
      lastSavedRef.current = new Date();
      onSuccess?.();
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));
      onError?.(err);
    } finally {
      isSavingRef.current = false;
    }
  }, [projectId, onSuccess, onError]);

  const markDirty = useCallback(() => {
    isDirtyRef.current = true;
  }, []);

  // Set up auto-save timer
  useEffect(() => {
    if (!enabled || !projectId) {
      return;
    }

    timerRef.current = setInterval(() => {
      if (isDirtyRef.current) {
        saveNow();
      }
    }, interval);

    return () => {
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
    };
  }, [enabled, projectId, interval, saveNow]);

  return {
    saveNow,
    markDirty,
    isSaving: isSavingRef.current,
    lastSaved: lastSavedRef.current,
  };
}
