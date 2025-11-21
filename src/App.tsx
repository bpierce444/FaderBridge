import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <div className="min-h-screen bg-slate-950 text-white flex items-center justify-center">
      <div className="container mx-auto p-8">
        <div className="text-center">
          <h1 className="text-4xl font-bold mb-4 text-cyan-500">
            Welcome to FaderBridge
          </h1>
          <p className="text-slate-400 mb-8">
            Professional MIDI-to-UCNet bridge for PreSonus hardware
          </p>

          <div className="max-w-md mx-auto space-y-4">
            <input
              className="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg focus:outline-none focus:border-cyan-500 text-white"
              onChange={(e) => setName(e.currentTarget.value)}
              placeholder="Enter a name..."
              value={name}
            />
            <button
              className="w-full px-6 py-3 bg-cyan-500 hover:bg-cyan-600 text-slate-950 font-semibold rounded-lg transition-colors"
              onClick={() => greet()}
            >
              Greet
            </button>
            {greetMsg && (
              <p className="text-cyan-400 mt-4">{greetMsg}</p>
            )}
          </div>

          <div className="mt-12 text-slate-500 text-sm">
            <p>Phase 1 MVP - Development Build</p>
            <p className="mt-2">
              Status: 0/7 tasks complete
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
