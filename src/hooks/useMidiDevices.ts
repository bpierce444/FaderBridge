import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { MidiDevice, MidiDeviceType } from '../types/midi';

/**
 * Hook for managing MIDI devices
 * Provides device discovery, connection management, and hot-plug detection
 */
export function useMidiDevices() {
  const [devices, setDevices] = useState<MidiDevice[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Discover all available MIDI devices
   */
  const discoverDevices = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const discoveredDevices = await invoke<MidiDevice[]>('discover_midi_devices');
      setDevices(discoveredDevices);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to discover MIDI devices:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Get cached MIDI devices
   */
  const getDevices = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const cachedDevices = await invoke<MidiDevice[]>('get_midi_devices');
      setDevices(cachedDevices);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to get MIDI devices:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Connect to a MIDI device
   */
  const connectDevice = useCallback(async (deviceId: string, deviceType: MidiDeviceType) => {
    setError(null);
    try {
      await invoke('connect_midi_device', { deviceId, deviceType });
      // Update local state
      setDevices(prev =>
        prev.map(device =>
          device.id === deviceId
            ? { ...device, status: 'connected' as const }
            : device
        )
      );
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to connect to MIDI device:', err);
      throw err;
    }
  }, []);

  /**
   * Disconnect from a MIDI device
   */
  const disconnectDevice = useCallback(async (deviceId: string, deviceType: MidiDeviceType) => {
    setError(null);
    try {
      await invoke('disconnect_midi_device', { deviceId, deviceType });
      // Update local state
      setDevices(prev =>
        prev.map(device =>
          device.id === deviceId
            ? { ...device, status: 'available' as const }
            : device
        )
      );
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to disconnect from MIDI device:', err);
      throw err;
    }
  }, []);

  /**
   * Check for device changes (hot-plug detection)
   */
  const checkForChanges = useCallback(async () => {
    try {
      const [added, removed] = await invoke<[MidiDevice[], MidiDevice[]]>(
        'check_midi_device_changes'
      );

      if (added.length > 0 || removed.length > 0) {
        // Refresh device list
        await discoverDevices();
        return { added, removed };
      }
      return { added: [], removed: [] };
    } catch (err) {
      console.error('Failed to check for MIDI device changes:', err);
      return { added: [], removed: [] };
    }
  }, [discoverDevices]);

  /**
   * Auto-discover devices on mount
   */
  useEffect(() => {
    discoverDevices();
  }, [discoverDevices]);

  /**
   * Set up hot-plug detection polling
   */
  useEffect(() => {
    const interval = setInterval(() => {
      checkForChanges();
    }, 2000); // Check every 2 seconds

    return () => clearInterval(interval);
  }, [checkForChanges]);

  return {
    devices,
    loading,
    error,
    discoverDevices,
    getDevices,
    connectDevice,
    disconnectDevice,
    checkForChanges,
  };
}
