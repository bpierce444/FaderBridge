/**
 * React hook for managing UCNet device discovery and connections
 */

import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { UcNetDevice } from "../types/ucnet";

export function useUcNetDevices() {
  const [devices, setDevices] = useState<UcNetDevice[]>([]);
  const [connectedDevices, setConnectedDevices] = useState<UcNetDevice[]>([]);
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Discovers UCNet devices on network and USB
   */
  const discoverDevices = useCallback(async () => {
    setIsDiscovering(true);
    setError(null);

    try {
      const discovered = await invoke<UcNetDevice[]>("discover_devices");
      setDevices(discovered);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Discovery failed: ${errorMessage}`);
      console.error("Device discovery error:", err);
    } finally {
      setIsDiscovering(false);
    }
  }, []);

  /**
   * Connects to a specific device
   */
  const connectDevice = useCallback(async (deviceId: string) => {
    setError(null);

    try {
      await invoke("connect_device", { deviceId });
      
      // Refresh connected devices list
      const connected = await invoke<UcNetDevice[]>("get_connected_devices");
      setConnectedDevices(connected);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Connection failed: ${errorMessage}`);
      console.error("Device connection error:", err);
      throw err;
    }
  }, []);

  /**
   * Disconnects from a specific device
   */
  const disconnectDevice = useCallback(async (deviceId: string) => {
    setError(null);

    try {
      await invoke("disconnect_device", { deviceId });
      
      // Refresh connected devices list
      const connected = await invoke<UcNetDevice[]>("get_connected_devices");
      setConnectedDevices(connected);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Disconnection failed: ${errorMessage}`);
      console.error("Device disconnection error:", err);
      throw err;
    }
  }, []);

  /**
   * Refreshes the list of connected devices
   */
  const refreshConnectedDevices = useCallback(async () => {
    try {
      const connected = await invoke<UcNetDevice[]>("get_connected_devices");
      setConnectedDevices(connected);
    } catch (err) {
      console.error("Failed to refresh connected devices:", err);
    }
  }, []);

  return {
    devices,
    connectedDevices,
    isDiscovering,
    error,
    discoverDevices,
    connectDevice,
    disconnectDevice,
    refreshConnectedDevices,
  };
}
