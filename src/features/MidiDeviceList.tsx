import { useMidiDevices } from '../hooks/useMidiDevices';
import type { MidiDevice } from '../types/midi';

/**
 * MIDI Device List Component
 * Displays all available MIDI devices with connection controls
 */
export function MidiDeviceList() {
  const { devices, loading, error, connectDevice, disconnectDevice, discoverDevices } =
    useMidiDevices();

  const handleConnect = async (device: MidiDevice) => {
    try {
      await connectDevice(device.id, device.device_type);
    } catch (err) {
      // Error is already handled in the hook
    }
  };

  const handleDisconnect = async (device: MidiDevice) => {
    try {
      await disconnectDevice(device.id, device.device_type);
    } catch (err) {
      // Error is already handled in the hook
    }
  };

  const inputDevices = devices.filter(d => d.device_type === 'input');
  const outputDevices = devices.filter(d => d.device_type === 'output');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold text-slate-100">MIDI Devices</h2>
        <button
          onClick={discoverDevices}
          disabled={loading}
          className="px-4 py-2 bg-cyan-600 hover:bg-cyan-700 disabled:bg-slate-700 disabled:text-slate-500 text-white rounded-md transition-colors"
        >
          {loading ? 'Scanning...' : 'Refresh'}
        </button>
      </div>

      {/* Error Display */}
      {error && (
        <div className="p-4 bg-red-900/20 border border-red-800 rounded-md">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      )}

      {/* Input Devices */}
      <div className="space-y-3">
        <h3 className="text-lg font-medium text-slate-300 flex items-center gap-2">
          <svg
            className="w-5 h-5 text-cyan-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
            />
          </svg>
          MIDI Inputs ({inputDevices.length})
        </h3>
        {inputDevices.length === 0 ? (
          <p className="text-slate-500 text-sm italic">No MIDI input devices found</p>
        ) : (
          <div className="space-y-2">
            {inputDevices.map(device => (
              <DeviceCard
                key={device.id}
                device={device}
                onConnect={handleConnect}
                onDisconnect={handleDisconnect}
              />
            ))}
          </div>
        )}
      </div>

      {/* Output Devices */}
      <div className="space-y-3">
        <h3 className="text-lg font-medium text-slate-300 flex items-center gap-2">
          <svg
            className="w-5 h-5 text-cyan-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
            />
          </svg>
          MIDI Outputs ({outputDevices.length})
        </h3>
        {outputDevices.length === 0 ? (
          <p className="text-slate-500 text-sm italic">No MIDI output devices found</p>
        ) : (
          <div className="space-y-2">
            {outputDevices.map(device => (
              <DeviceCard
                key={device.id}
                device={device}
                onConnect={handleConnect}
                onDisconnect={handleDisconnect}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Individual device card component
 */
function DeviceCard({
  device,
  onConnect,
  onDisconnect,
}: {
  device: MidiDevice;
  onConnect: (device: MidiDevice) => void;
  onDisconnect: (device: MidiDevice) => void;
}) {
  const isConnected = device.status === 'connected';

  return (
    <div className="p-4 bg-slate-800 border border-slate-700 rounded-md hover:border-slate-600 transition-colors">
      <div className="flex items-center justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3">
            {/* Status Indicator */}
            <div
              className={`w-3 h-3 rounded-full ${
                isConnected
                  ? 'bg-cyan-400 shadow-lg shadow-cyan-500/50'
                  : 'bg-slate-600'
              }`}
            />
            
            {/* Device Info */}
            <div className="flex-1 min-w-0">
              <h4 className="text-slate-100 font-medium truncate">{device.name}</h4>
              {device.manufacturer && (
                <p className="text-slate-400 text-sm">{device.manufacturer}</p>
              )}
              <p className="text-slate-500 text-xs">Port {device.port_number}</p>
            </div>
          </div>
        </div>

        {/* Connect/Disconnect Button */}
        <button
          onClick={() => (isConnected ? onDisconnect(device) : onConnect(device))}
          className={`px-4 py-2 rounded-md font-medium transition-colors ${
            isConnected
              ? 'bg-slate-700 hover:bg-slate-600 text-slate-300'
              : 'bg-cyan-600 hover:bg-cyan-700 text-white'
          }`}
        >
          {isConnected ? 'Disconnect' : 'Connect'}
        </button>
      </div>
    </div>
  );
}
