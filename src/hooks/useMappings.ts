/**
 * React hook for managing parameter mappings
 * Provides CRUD operations for mappings in the current project
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

/**
 * Taper curve type for parameter mapping
 */
export type TaperCurve = 'linear' | 'logarithmic' | 'exponential' | 's-curve';

/**
 * Device type enumeration
 */
export type DeviceType = 'midi' | 'ucnet';

/**
 * Represents a device configuration
 */
export interface Device {
  id: number;
  project_id: number;
  device_type: DeviceType;
  device_name: string;
  device_id: string;
  connection_type: string | null;
  config_json: string | null;
  created_at: string;
}

/**
 * Represents a parameter mapping
 */
export interface Mapping {
  id: number;
  project_id: number;
  midi_device_id: number;
  ucnet_device_id: number;
  midi_channel: number;
  midi_cc: number;
  ucnet_parameter_id: number;
  ucnet_parameter_name: string;
  taper_curve: TaperCurve;
  min_value: number;
  max_value: number;
  invert: boolean;
  bidirectional: boolean;
  label: string | null;
  created_at: string;
}

/**
 * Request to create a new mapping
 */
export interface CreateMappingRequest {
  project_id: number;
  midi_device_id: number;
  ucnet_device_id: number;
  midi_channel: number;
  midi_cc: number;
  ucnet_parameter_id: number;
  ucnet_parameter_name: string;
  taper_curve?: TaperCurve;
  min_value?: number;
  max_value?: number;
  invert?: boolean;
  bidirectional?: boolean;
  label?: string;
}

/**
 * Request to update an existing mapping
 */
export interface UpdateMappingRequest {
  id: number;
  taper_curve?: TaperCurve;
  min_value?: number;
  max_value?: number;
  invert?: boolean;
  bidirectional?: boolean;
  label?: string;
}

export interface UseMappingsReturn {
  /** All mappings in the current project */
  mappings: Mapping[];
  /** All devices in the current project */
  devices: Device[];
  /** Whether data is currently loading */
  loading: boolean;
  /** Error message if any */
  error: string | null;
  /** Create a new mapping */
  createMapping: (req: CreateMappingRequest) => Promise<Mapping | null>;
  /** Update an existing mapping */
  updateMapping: (req: UpdateMappingRequest) => Promise<Mapping | null>;
  /** Delete a mapping by ID */
  deleteMapping: (id: number) => Promise<boolean>;
  /** Refresh mappings and devices from backend */
  refresh: () => Promise<void>;
}

/**
 * Hook for managing parameter mappings in the current project
 * 
 * @param projectId - The ID of the project to manage mappings for
 * @returns Mappings state and CRUD operations
 * 
 * @example
 * ```tsx
 * const { mappings, devices, createMapping, deleteMapping } = useMappings(1);
 * 
 * // Create a new mapping
 * await createMapping({
 *   project_id: 1,
 *   midi_device_id: 1,
 *   ucnet_device_id: 2,
 *   midi_channel: 0,
 *   midi_cc: 7,
 *   ucnet_parameter_id: 100,
 *   ucnet_parameter_name: "Channel 1 Volume",
 * });
 * 
 * // Delete a mapping
 * await deleteMapping(mappingId);
 * ```
 */
export function useMappings(projectId: number | null): UseMappingsReturn {
  const [mappings, setMappings] = useState<Mapping[]>([]);
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetches mappings and devices from the backend
   */
  const refresh = useCallback(async () => {
    if (projectId === null) {
      setMappings([]);
      setDevices([]);
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const [fetchedMappings, fetchedDevices] = await Promise.all([
        invoke<Mapping[]>('get_mappings_by_project', { projectId }),
        invoke<Device[]>('get_devices_by_project', { projectId }),
      ]);

      setMappings(fetchedMappings);
      setDevices(fetchedDevices);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  /**
   * Creates a new mapping
   */
  const createMapping = useCallback(
    async (req: CreateMappingRequest): Promise<Mapping | null> => {
      try {
        setError(null);
        const mapping = await invoke<Mapping>('create_mapping', { req });
        await refresh();
        return mapping;
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        return null;
      }
    },
    [refresh]
  );

  /**
   * Updates an existing mapping
   */
  const updateMapping = useCallback(
    async (req: UpdateMappingRequest): Promise<Mapping | null> => {
      try {
        setError(null);
        const mapping = await invoke<Mapping>('update_mapping', { req });
        await refresh();
        return mapping;
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        return null;
      }
    },
    [refresh]
  );

  /**
   * Deletes a mapping by ID
   */
  const deleteMapping = useCallback(
    async (id: number): Promise<boolean> => {
      try {
        setError(null);
        await invoke('delete_mapping', { id });
        await refresh();
        return true;
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        return false;
      }
    },
    [refresh]
  );

  /**
   * Initial data fetch
   */
  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    mappings,
    devices,
    loading,
    error,
    createMapping,
    updateMapping,
    deleteMapping,
    refresh,
  };
}
