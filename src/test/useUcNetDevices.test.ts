/**
 * Tests for useUcNetDevices hook
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useUcNetDevices } from "../hooks/useUcNetDevices";
import type { UcNetDevice } from "../types/ucnet";

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

describe("useUcNetDevices", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  it("should initialize with empty state", () => {
    const { result } = renderHook(() => useUcNetDevices());

    expect(result.current.devices).toEqual([]);
    expect(result.current.connectedDevices).toEqual([]);
    expect(result.current.isDiscovering).toBe(false);
    expect(result.current.error).toBeNull();
  });

  it("should discover devices successfully", async () => {
    const mockDevices: UcNetDevice[] = [
      {
        id: "net-test-001",
        model: "StudioLive 32SX",
        firmware_version: "1.0.0",
        connection_type: "Network",
        state: "Discovered",
        identifier: "192.168.1.100",
      },
    ];

    mockInvoke.mockResolvedValueOnce(mockDevices);

    const { result } = renderHook(() => useUcNetDevices());

    await act(async () => {
      await result.current.discoverDevices();
    });

    await waitFor(() => {
      expect(result.current.devices).toEqual(mockDevices);
      expect(result.current.isDiscovering).toBe(false);
      expect(result.current.error).toBeNull();
    });

    expect(mockInvoke).toHaveBeenCalledWith("discover_devices");
  });

  it("should handle discovery errors", async () => {
    const errorMessage = "Network error";
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage));

    const { result } = renderHook(() => useUcNetDevices());

    await act(async () => {
      await result.current.discoverDevices();
    });

    await waitFor(() => {
      expect(result.current.error).toContain(errorMessage);
      expect(result.current.isDiscovering).toBe(false);
    });
  });

  it("should connect to a device successfully", async () => {
    const mockConnectedDevices: UcNetDevice[] = [
      {
        id: "net-test-001",
        model: "StudioLive 32SX",
        firmware_version: "1.0.0",
        connection_type: "Network",
        state: "Connected",
        identifier: "192.168.1.100",
      },
    ];

    mockInvoke
      .mockResolvedValueOnce(undefined) // connect_device
      .mockResolvedValueOnce(mockConnectedDevices); // get_connected_devices

    const { result } = renderHook(() => useUcNetDevices());

    await act(async () => {
      await result.current.connectDevice("net-test-001");
    });

    await waitFor(() => {
      expect(result.current.connectedDevices).toEqual(mockConnectedDevices);
      expect(result.current.error).toBeNull();
    });

    expect(mockInvoke).toHaveBeenCalledWith("connect_device", {
      deviceId: "net-test-001",
    });
    expect(mockInvoke).toHaveBeenCalledWith("get_connected_devices");
  });

  it("should disconnect from a device successfully", async () => {
    mockInvoke
      .mockResolvedValueOnce(undefined) // disconnect_device
      .mockResolvedValueOnce([]); // get_connected_devices

    const { result } = renderHook(() => useUcNetDevices());

    await act(async () => {
      await result.current.disconnectDevice("net-test-001");
    });

    await waitFor(() => {
      expect(result.current.connectedDevices).toEqual([]);
      expect(result.current.error).toBeNull();
    });

    expect(mockInvoke).toHaveBeenCalledWith("disconnect_device", {
      deviceId: "net-test-001",
    });
  });

  it("should handle connection errors", async () => {
    const errorMessage = "Connection failed";
    mockInvoke.mockRejectedValueOnce(new Error(errorMessage));

    const { result } = renderHook(() => useUcNetDevices());

    await act(async () => {
      try {
        await result.current.connectDevice("net-test-001");
      } catch {
        // Expected to throw
      }
    });

    await waitFor(() => {
      expect(result.current.error).toContain(errorMessage);
    });
  });
});
