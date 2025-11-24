/**
 * Dashboard component
 * Main application dashboard integrating all Phase 1 features
 */

import { useEffect } from 'react';
import { useProjects } from '../hooks/useProjects';
import { Layout } from '../components/Layout';
import { TopBar } from '../components/TopBar';
import { StatusBar } from '../components/StatusBar';
import { MidiDeviceList } from './MidiDeviceList';
import { DeviceManager } from './DeviceManager';
import { MappingManager } from './MappingManager';

/**
 * Main dashboard component that integrates all Phase 1 MVP features
 * Follows PRD Section 5 layout: Left (MIDI) / Center (Mappings) / Right (UCNet)
 * 
 * @example
 * ```tsx
 * <Dashboard />
 * ```
 */
export function Dashboard() {
  const { activeProject } = useProjects();

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+S / Ctrl+S to save
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault();
        // Save is handled by auto-save, but we can trigger immediate save if needed
        console.log('Save shortcut triggered');
      }

      // ESC to cancel/close dialogs (handled by individual components)
      if (e.key === 'Escape') {
        // Let components handle their own ESC logic
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  return (
    <Layout
      topBar={<TopBar />}
      leftPanel={
        <div className="space-y-4">
          <div>
            <h2 className="text-base font-semibold text-white mb-2">MIDI Devices</h2>
            <p className="text-xs text-slate-400 mb-3">
              Control surfaces with physical faders/buttons
            </p>
          </div>
          <MidiDeviceList />
        </div>
      }
      centerPanel={
        <div className="space-y-4">
          {!activeProject ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center max-w-md">
                <svg
                  className="w-20 h-20 text-slate-700 mx-auto mb-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
                <h3 className="text-lg font-semibold text-white mb-2">Welcome to FaderBridge</h3>
                <p className="text-sm text-slate-400 mb-4">
                  Create or load a project to start mapping MIDI controls to UCNet parameters
                </p>
                <div className="text-xs text-slate-500">
                  Click <span className="text-cyan-400 font-medium">New</span> in the top bar to get started
                </div>
              </div>
            </div>
          ) : (
            <MappingManager projectId={activeProject.id} />
          )}
        </div>
      }
      rightPanel={
        <div className="space-y-4">
          <div>
            <h2 className="text-base font-semibold text-white mb-2">UCNet Devices</h2>
            <p className="text-xs text-slate-400 mb-3">
              Connect to PreSonus hardware
            </p>
          </div>
          <DeviceManager />
        </div>
      }
      statusBar={
        <StatusBar
          projectId={activeProject?.id ?? null}
          showMidiActivity={true}
          showUCNetActivity={true}
        />
      }
    />
  );
}
