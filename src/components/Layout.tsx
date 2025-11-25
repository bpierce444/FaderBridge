/**
 * Layout component
 * Main application layout with Left/Center/Right panel structure
 */

import { ReactNode } from 'react';

export interface LayoutProps {
  /** Top bar content (project management, etc.) */
  topBar?: ReactNode;
  /** Left panel content (MIDI devices) */
  leftPanel: ReactNode;
  /** Center panel content (mapping interface) */
  centerPanel: ReactNode;
  /** Right panel content (UCNet devices) */
  rightPanel: ReactNode;
  /** Bottom status bar content */
  statusBar?: ReactNode;
}

/**
 * Main application layout following PRD Section 5 guidelines
 * Implements a three-panel dashboard with top bar and status bar
 * 
 * @example
 * ```tsx
 * <Layout
 *   topBar={<TopBar />}
 *   leftPanel={<MidiDeviceList />}
 *   centerPanel={<MappingManager />}
 *   rightPanel={<DeviceManager />}
 *   statusBar={<StatusBar />}
 * />
 * ```
 */
export function Layout({
  topBar,
  leftPanel,
  centerPanel,
  rightPanel,
  statusBar,
}: LayoutProps) {
  return (
    <div className="h-screen flex flex-col bg-slate-950 text-white">
      {/* Top Bar */}
      {topBar && (
        <div className="flex-shrink-0 border-b border-slate-800">
          {topBar}
        </div>
      )}

      {/* Main Content Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Panel - MIDI Devices */}
        <div className="w-72 min-w-[280px] flex-shrink-0 border-r border-slate-800 overflow-y-auto">
          <div className="p-4">
            {leftPanel}
          </div>
        </div>

        {/* Center Panel - Mapping Interface */}
        <div className="flex-1 overflow-y-auto">
          <div className="p-6">
            {centerPanel}
          </div>
        </div>

        {/* Right Panel - UCNet Devices */}
        <div className="w-80 min-w-[320px] flex-shrink-0 border-l border-slate-800 overflow-y-auto overflow-x-hidden">
          <div className="p-4">
            {rightPanel}
          </div>
        </div>
      </div>

      {/* Status Bar */}
      {statusBar && (
        <div className="flex-shrink-0 border-t border-slate-800">
          {statusBar}
        </div>
      )}
    </div>
  );
}
