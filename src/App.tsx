import { DeviceManager } from "./features/DeviceManager";
import { MidiDeviceList } from "./features/MidiDeviceList";

function App() {
  return (
    <div className="min-h-screen bg-slate-950 text-white">
      <div className="container mx-auto p-8">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl font-bold mb-4 text-cyan-500">
            FaderBridge
          </h1>
          <p className="text-slate-400">
            Professional MIDI-to-UCNet bridge for PreSonus hardware
          </p>
        </div>

        {/* Main Content */}
        <div className="max-w-6xl mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* MIDI Devices */}
            <div className="bg-slate-900 p-6 rounded-lg border border-slate-800">
              <MidiDeviceList />
            </div>

            {/* UCNet Devices */}
            <div className="bg-slate-900 p-6 rounded-lg border border-slate-800">
              <DeviceManager />
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="mt-12 text-center text-slate-500 text-sm">
          <p>Phase 1 MVP - Development Build</p>
          <p className="mt-2">
            Status: 7/7 tasks complete ✅ | MCU Protocol Support Added
          </p>
        </div>
      </div>
    </div>
  );
}

export default App;
