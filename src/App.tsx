import { DeviceManager } from "./features/DeviceManager";

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
        <div className="max-w-4xl mx-auto">
          <DeviceManager />
        </div>

        {/* Footer */}
        <div className="mt-12 text-center text-slate-500 text-sm">
          <p>Phase 1 MVP - Development Build</p>
          <p className="mt-2">
            Status: 1/7 tasks complete (TASK-001: UCNet Device Discovery)
          </p>
        </div>
      </div>
    </div>
  );
}

export default App;
