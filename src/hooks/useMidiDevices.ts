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
      
      // Start MIDI monitoring if any input devices are connected
      const hasConnectedInput = discoveredDevices.some(
        d => d.device_type === 'input' && d.status === 'connected'
      );
      if (hasConnectedInput) {
        try {
          await invoke('start_midi_monitoring');
        } catch (monitorErr) {
          console.warn('Failed to start MIDI monitoring:', monitorErr);
        }
      }
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
    console.log('useMidiDevices.connectDevice called:', deviceId, deviceType);
    setError(null);
    try {
      // Start MIDI monitoring BEFORE connecting input devices
      // This ensures the message channel is set up before the connection callback is created
      if (deviceType === 'input') {
        try {
          console.log('Starting MIDI monitoring first...');
          await invoke('start_midi_monitoring');
          console.log('MIDI monitoring started');
        } catch (monitorErr) {
          console.warn('Failed to start MIDI monitoring:', monitorErr);
          // Don't fail the connection if monitoring fails
        }
      }
      
      console.log('Invoking connect_midi_device...');
      await invoke('connect_midi_device', { deviceId, deviceType });
      console.log('connect_midi_device succeeded');
      
      // Update local state
      setDevices(prev =>
        prev.map(device =>
          device.id === deviceId
            ? { ...device, status: 'connected' as const }
            : device
        )
      );
      console.log('Local state updated');
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
   * Hot-plug detection polling disabled on macOS
   * Creating multiple MidiInput instances causes CoreMIDI to fail
   * when retrieving port names. Users can click Refresh manually.
   * TODO: Re-enable with a macOS-compatible approach (e.g., CoreMIDI notifications)
   */
  // useEffect(() => {
  //   const interval = setInterval(() => {
  //     checkForChanges();
  //   }, 2000); // Check every 2 seconds
  //
  //   return () => clearInterval(interval);
  // }, [checkForChanges]);

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
