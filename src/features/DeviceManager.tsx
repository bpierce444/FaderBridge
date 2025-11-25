/**
 * DeviceManager Component
 * 
 * Displays discovered UCNet devices and manages connections.
 * Follows the "Dark Room Standard" design from STYLE_GUIDE.md
 */

import { useEffect } from "react";
import { useUcNetDevices } from "../hooks/useUcNetDevices";
import type { UcNetDevice, ConnectionState } from "../types/ucnet";

export function DeviceManager() {
  const {
    devices,
    connectedDevices,
    isDiscovering,
    error,
    discoverDevices,
    connectDevice,
    disconnectDevice,
    refreshConnectedDevices,
  } = useUcNetDevices();

  // Auto-discover on mount
  useEffect(() => {
    discoverDevices();
    refreshConnectedDevices();
  }, [discoverDevices, refreshConnectedDevices]);

  const isDeviceConnected = (deviceId: string): boolean => {
    return connectedDevices.some((d) => d.id === deviceId);
  };

  const handleToggleConnection = async (device: UcNetDevice) => {
    try {
      if (isDeviceConnected(device.id)) {
        await disconnectDevice(device.id);
      } else {
        await connectDevice(device.id);
      }
    } catch (err) {
      // Error is already handled in the hook
      console.error("Connection toggle failed:", err);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-white">Mixers & Interfaces</h2>
          <p className="text-slate-400 text-xs mt-0.5">
            PreSonus UCNet devices (Series III, Quantum)
          </p>
        </div>
        
        <button
          onClick={discoverDevices}
          disabled={isDiscovering}
          className="px-3 py-1.5 text-sm bg-cyan-500 hover:bg-cyan-600 disabled:bg-slate-700 disabled:text-slate-500 text-slate-950 font-medium rounded-md transition-colors"
        >
          {isDiscovering ? "Discovering..." : "Refresh"}
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900/20 border border-red-500/50 rounded-lg">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      )}

      {/* Device List */}
      <div className="space-y-3">
        {devices.length === 0 && !isDiscovering && (
          <div className="p-8 text-center bg-slate-900/50 border border-slate-800 rounded-lg">
            <p className="text-slate-400">No mixers or interfaces found</p>
            <p className="text-slate-500 text-sm mt-2">
              Connect your Series III mixer or Quantum interface via USB or Thunderbolt
            </p>
          </div>
        )}

        {devices.map((device) => (
          <DeviceCard
            key={device.id}
            device={device}
            isConnected={isDeviceConnected(device.id)}
            onToggleConnection={() => handleToggleConnection(device)}
          />
        ))}
      </div>

      {/* Connected Devices Summary */}
      {connectedDevices.length > 0 && (
        <div className="p-4 bg-cyan-900/20 border border-cyan-500/30 rounded-lg">
          <p className="text-cyan-400 text-sm">
            {connectedDevices.length} device{connectedDevices.length !== 1 ? "s" : ""} connected
          </p>
        </div>
      )}
    </div>
  );
}

interface DeviceCardProps {
  device: UcNetDevice;
  isConnected: boolean;
  onToggleConnection: () => void;
}

function DeviceCard({ device, isConnected, onToggleConnection }: DeviceCardProps) {
  return (
    <div className="p-3 bg-slate-900 border border-slate-800 rounded-lg hover:border-slate-700 transition-colors">
      <div className="flex items-center gap-3">
        {/* Connection Status Indicator */}
        <ConnectionStatusIndicator state={device.state} />
        
        {/* Device Info - truncate long text */}
        <div className="flex-1 min-w-0">
          <h3 className="text-white text-sm font-medium truncate">{device.model}</h3>
          <div className="flex items-center gap-1.5 mt-0.5 text-xs text-slate-400">
            <ConnectionTypeIcon type={device.connection_type} />
            <span className="truncate">{device.identifier}</span>
            <span className="flex-shrink-0">• v{device.firmware_version}</span>
          </div>
        </div>

        {/* Connect/Disconnect Button - fixed width to prevent cutoff */}
        <button
          onClick={onToggleConnection}
          className={`flex-shrink-0 px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
            isConnected
              ? "bg-slate-700 hover:bg-slate-600 text-white"
              : "bg-cyan-500 hover:bg-cyan-600 text-slate-950"
          }`}
        >
          {isConnected ? "Disconnect" : "Connect"}
        </button>
      </div>
    </div>
  );
}

function ConnectionStatusIndicator({ state }: { state: ConnectionState }) {
  const colors = {
    Discovered: "bg-slate-500",
    Connecting: "bg-yellow-500 animate-pulse",
    Connected: "bg-green-500",
    Disconnected: "bg-slate-600",
    Failed: "bg-red-500",
  };

  return (
    <div className={`w-3 h-3 rounded-full ${colors[state]}`} title={state} />
  );
}

function ConnectionTypeIcon({ type }: { type: "Network" | "Usb" }) {
  if (type === "Network") {
    return (
      <svg
        className="w-4 h-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
        />
      </svg>
    );
  }

  return (
    <svg
      className="w-4 h-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
      />
    </svg>
  );
}
