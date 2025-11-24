/**
 * StatusBar component
 * Bottom status bar showing sync status, latency metrics, and activity indicators
 */

import { SyncStatusIndicator } from './SyncStatusIndicator';
import { ActivityLight } from './ActivityLight';
import { useAutoSave } from '../hooks/useAutoSave';

export interface StatusBarProps {
  /** Current project ID for auto-save indicator */
  projectId: number | null;
  /** Show MIDI activity indicator */
  showMidiActivity?: boolean;
  /** Show UCNet activity indicator */
  showUCNetActivity?: boolean;
}

/**
 * Status bar with sync status, latency metrics, and activity indicators
 * Displayed at the bottom of the application
 * 
 * @example
 * ```tsx
 * <StatusBar 
 *   projectId={activeProject?.id ?? null}
 *   showMidiActivity={true}
 *   showUCNetActivity={true}
 * />
 * ```
 */
export function StatusBar({
  projectId,
  showMidiActivity = true,
  showUCNetActivity = true,
}: StatusBarProps) {
  const { isSaving, lastSaved } = useAutoSave({ projectId });

  return (
    <div className="h-12 px-6 flex items-center justify-between bg-slate-900">
      {/* Left: Sync Status */}
      <div className="flex items-center gap-6">
        <SyncStatusIndicator showDetails={false} />
        
        {/* Activity Indicators */}
        <div className="flex items-center gap-4">
          {showMidiActivity && (
            <div className="flex items-center gap-2">
              <ActivityLight active={false} size={8} />
              <span className="text-xs text-slate-400">MIDI</span>
            </div>
          )}
          {showUCNetActivity && (
            <div className="flex items-center gap-2">
              <ActivityLight active={false} size={8} />
              <span className="text-xs text-slate-400">UCNet</span>
            </div>
          )}
        </div>
      </div>

      {/* Right: Auto-save Status */}
      <div className="flex items-center gap-4">
        {projectId && (
          <div className="flex items-center gap-2 text-xs text-slate-400">
            {isSaving ? (
              <>
                <div className="w-2 h-2 rounded-full bg-cyan-500 animate-pulse"></div>
                <span>Saving...</span>
              </>
            ) : lastSaved ? (
              <>
                <div className="w-2 h-2 rounded-full bg-emerald-500"></div>
                <span>Saved {formatTimeSince(lastSaved)}</span>
              </>
            ) : null}
          </div>
        )}
        
        {/* Version Info */}
        <div className="text-xs text-slate-500">
          Phase 1 MVP
        </div>
      </div>
    </div>
  );
}

/**
 * Format time since last save in a human-readable format
 */
function formatTimeSince(date: Date): string {
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  
  if (seconds < 60) {
    return 'just now';
  }
  
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }
  
  return 'today';
}
