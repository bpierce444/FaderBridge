import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useMidiDevices } from './useMidiDevices';
import type { MidiDevice } from '../types/midi';

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe('useMidiDevices', () => {
  const mockDevices: MidiDevice[] = [
    {
      id: 'input:0:Test Input',
      name: 'Test Input',
      manufacturer: 'Test',
      device_type: 'input',
      port_number: 0,
      status: 'available',
    },
    {
      id: 'output:0:Test Output',
      name: 'Test Output',
      manufacturer: 'Test',
      device_type: 'output',
      port_number: 0,
      status: 'available',
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue(mockDevices);
  });

  it('should discover devices on mount', async () => {
    const { result } = renderHook(() => useMidiDevices());

    expect(result.current.loading).toBe(true);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(mockInvoke).toHaveBeenCalledWith('discover_midi_devices');
    expect(result.current.devices).toEqual(mockDevices);
    expect(result.current.error).toBeNull();
  });

  it('should handle discovery errors', async () => {
    mockInvoke.mockRejectedValueOnce(new Error('Discovery failed'));

    const { result } = renderHook(() => useMidiDevices());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe('Discovery failed');
    expect(result.current.devices).toEqual([]);
  });

  it('should connect to a device', async () => {
    mockInvoke.mockResolvedValueOnce(mockDevices);
    mockInvoke.mockResolvedValueOnce(undefined); // connect response

    const { result } = renderHook(() => useMidiDevices());

    await waitFor(() => {
      expect(result.current.devices.length).toBe(2);
    });

    await result.current.connectDevice('input:0:Test Input', 'input');

    expect(mockInvoke).toHaveBeenCalledWith('connect_midi_device', {
      deviceId: 'input:0:Test Input',
      deviceType: 'input',
    });

    // Wait for state to update
    await waitFor(() => {
      const connectedDevice = result.current.devices.find(d => d.id === 'input:0:Test Input');
      expect(connectedDevice?.status).toBe('connected');
    });
  });

  it('should disconnect from a device', async () => {
    const connectedDevices = mockDevices.map(d => ({ ...d, status: 'connected' as const }));
    mockInvoke.mockResolvedValueOnce(connectedDevices);
    mockInvoke.mockResolvedValueOnce(undefined); // disconnect response

    const { result } = renderHook(() => useMidiDevices());

    await waitFor(() => {
      expect(result.current.devices.length).toBe(2);
    });

    await result.current.disconnectDevice('input:0:Test Input', 'input');

    expect(mockInvoke).toHaveBeenCalledWith('disconnect_midi_device', {
      deviceId: 'input:0:Test Input',
      deviceType: 'input',
    });

    // Wait for state to update
    await waitFor(() => {
      const disconnectedDevice = result.current.devices.find(d => d.id === 'input:0:Test Input');
      expect(disconnectedDevice?.status).toBe('available');
    });
  });

  it('should handle connection errors', async () => {
    mockInvoke.mockResolvedValueOnce(mockDevices);
    mockInvoke.mockRejectedValueOnce(new Error('Connection failed'));

    const { result } = renderHook(() => useMidiDevices());

    await waitFor(() => {
      expect(result.current.devices.length).toBe(2);
    });

    await expect(
      result.current.connectDevice('input:0:Test Input', 'input')
    ).rejects.toThrow('Connection failed');

    // Wait for error state to update
    await waitFor(() => {
      expect(result.current.error).toBe('Connection failed');
    });
  });

  it('should check for device changes', async () => {
    const addedDevice: MidiDevice = {
      id: 'input:1:New Device',
      name: 'New Device',
      manufacturer: 'New',
      device_type: 'input',
      port_number: 1,
      status: 'available',
    };

    mockInvoke.mockResolvedValueOnce(mockDevices); // initial discovery
    mockInvoke.mockResolvedValueOnce([[addedDevice], []]); // check_for_changes
    mockInvoke.mockResolvedValueOnce([...mockDevices, addedDevice]); // re-discovery

    const { result } = renderHook(() => useMidiDevices());

    await waitFor(() => {
      expect(result.current.devices.length).toBe(2);
    });

    const changes = await result.current.checkForChanges();

    expect(changes.added).toEqual([addedDevice]);
    expect(changes.removed).toEqual([]);

    await waitFor(() => {
      expect(result.current.devices.length).toBe(3);
    });
  });

  it('should get cached devices', async () => {
    mockInvoke.mockResolvedValueOnce(mockDevices); // initial discovery
    mockInvoke.mockResolvedValueOnce(mockDevices); // get_devices

    const { result } = renderHook(() => useMidiDevices());

    await waitFor(() => {
      expect(result.current.devices.length).toBe(2);
    });

    await result.current.getDevices();

    expect(mockInvoke).toHaveBeenCalledWith('get_midi_devices');
  });
});
