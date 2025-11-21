/**
 * React hook for bidirectional sync operations
 * 
 * Provides access to the sync engine for managing parameter mappings,
 * monitoring latency, and handling state synchronization.
 */

import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import type { ParameterMapping } from '../types/mapping';

/**
 * Latency statistics
 */
export interface LatencyStats {
  avg_ms: number;
  min_ms: number;
  max_ms: number;
  sample_count: number;
}

/**
 * Request for adding a parameter mapping
 */
export interface AddMappingRequest {
  midi_channel: number;
  midi_controller?: number;
  midi_note?: number;
  ucnet_device_id: string;
  ucnet_channel: number;
  parameter_type: 'volume' | 'pan' | 'mute';
  taper_curve?: 'linear' | 'logarithmic' | 'audio';
  use_14bit?: boolean;
  midi_controller_msb?: number;
  midi_controller_lsb?: number;
}

/**
 * Hook for sync operations
 */
export function useSync() {
  const [isInitialized, setIsInitialized] = useState(false);
  const [latencyStats, setLatencyStats] = useState<LatencyStats | null>(null);
  const [mappings, setMappings] = useState<ParameterMapping[]>([]);

  /**
   * Initialize the sync engine
   */
  const initSyncEngine = useCallback(async () => {
    try {
      await invoke('init_sync_engine');
      setIsInitialized(true);
    } catch (error) {
      console.error('Failed to initialize sync engine:', error);
      throw error;
    }
  }, []);

  /**
   * Add a parameter mapping
   */
  const addMapping = useCallback(async (request: AddMappingRequest) => {
    try {
      await invoke('add_parameter_mapping', { request });
      // Refresh mappings
      await refreshMappings();
    } catch (error) {
      console.error('Failed to add parameter mapping:', error);
      throw error;
    }
  }, []);

  /**
   * Remove a parameter mapping
   */
  const removeMapping = useCallback(
    async (
      midiChannel: number,
      midiController?: number,
      midiNote?: number
    ) => {
      try {
        await invoke('remove_parameter_mapping', {
          midi_channel: midiChannel,
          midi_controller: midiController,
          midi_note: midiNote,
        });
        // Refresh mappings
        await refreshMappings();
      } catch (error) {
        console.error('Failed to remove parameter mapping:', error);
        throw error;
      }
    },
    []
  );

  /**
   * Clear all parameter mappings
   */
  const clearMappings = useCallback(async () => {
    try {
      await invoke('clear_parameter_mappings');
      setMappings([]);
    } catch (error) {
      console.error('Failed to clear parameter mappings:', error);
      throw error;
    }
  }, []);

  /**
   * Refresh parameter mappings from backend
   */
  const refreshMappings = useCallback(async () => {
    try {
      const result = await invoke<ParameterMapping[]>('get_parameter_mappings');
      setMappings(result);
    } catch (error) {
      console.error('Failed to get parameter mappings:', error);
      throw error;
    }
  }, []);

  /**
   * Get latency statistics
   */
  const getLatencyStats = useCallback(async () => {
    try {
      const stats = await invoke<LatencyStats | null>('get_latency_stats');
      setLatencyStats(stats);
      return stats;
    } catch (error) {
      console.error('Failed to get latency stats:', error);
      throw error;
    }
  }, []);

  /**
   * Clear latency statistics
   */
  const clearLatencyStats = useCallback(async () => {
    try {
      await invoke('clear_latency_stats');
      setLatencyStats(null);
    } catch (error) {
      console.error('Failed to clear latency stats:', error);
      throw error;
    }
  }, []);

  /**
   * Clear shadow state for a specific device
   */
  const clearDeviceState = useCallback(async (deviceId: string) => {
    try {
      await invoke('clear_device_state', { device_id: deviceId });
    } catch (error) {
      console.error('Failed to clear device state:', error);
      throw error;
    }
  }, []);

  /**
   * Clear all shadow state
   */
  const clearAllState = useCallback(async () => {
    try {
      await invoke('clear_all_state');
    } catch (error) {
      console.error('Failed to clear all state:', error);
      throw error;
    }
  }, []);

  /**
   * Auto-initialize on mount
   */
  useEffect(() => {
    if (!isInitialized) {
      initSyncEngine().catch(console.error);
    }
  }, [isInitialized, initSyncEngine]);

  /**
   * Periodically refresh latency stats (every 2 seconds)
   */
  useEffect(() => {
    if (!isInitialized) return;

    const interval = setInterval(() => {
      getLatencyStats().catch(console.error);
    }, 2000);

    return () => clearInterval(interval);
  }, [isInitialized, getLatencyStats]);

  return {
    isInitialized,
    latencyStats,
    mappings,
    initSyncEngine,
    addMapping,
    removeMapping,
    clearMappings,
    refreshMappings,
    getLatencyStats,
    clearLatencyStats,
    clearDeviceState,
    clearAllState,
  };
}
