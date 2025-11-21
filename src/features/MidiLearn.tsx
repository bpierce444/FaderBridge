/**
 * MidiLearn feature component
 * Displays MIDI Learn status and provides controls
 */

import { useMidiLearn } from '../hooks/useMidiLearn';
import { getRemainingSeconds } from '../types/learn';

export interface MidiLearnProps {
  /** Optional CSS class name */
  className?: string;
}

/**
 * Component that displays MIDI Learn status and provides visual feedback
 * 
 * @example
 * ```tsx
 * <MidiLearn />
 * ```
 */
export function MidiLearn({ className = '' }: MidiLearnProps) {
  const { learnState, isLearning, cancelLearn, error } = useMidiLearn();

  if (!isLearning && !error) {
    return null;
  }

  const remainingSeconds = getRemainingSeconds(learnState);
  const progressPercent = isLearning ? (remainingSeconds / 10) * 100 : 0;

  return (
    <div
      className={`fixed bottom-4 right-4 bg-slate-800 border-2 border-amber-500 rounded-lg shadow-2xl p-4 min-w-[320px] ${className}`}
      role="alert"
      aria-live="polite"
    >
      {error && (
        <div className="bg-rose-900/50 border border-rose-600 rounded p-3 mb-3">
          <div className="flex items-start gap-2">
            <svg
              className="w-5 h-5 text-rose-500 flex-shrink-0 mt-0.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <div className="flex-1">
              <h3 className="text-rose-400 font-medium text-sm">Error</h3>
              <p className="text-rose-300 text-xs mt-1">{error}</p>
            </div>
          </div>
        </div>
      )}

      {isLearning && learnState.type === 'listening' && (
        <>
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 bg-amber-500 rounded-full animate-pulse" />
              <h3 className="text-amber-400 font-medium text-sm">MIDI Learn Active</h3>
            </div>
            <button
              onClick={cancelLearn}
              className="text-slate-400 hover:text-white transition-colors"
              title="Cancel (ESC)"
              aria-label="Cancel MIDI Learn"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          <div className="space-y-2 mb-3">
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Device:</span>
              <span className="text-white font-mono">{learnState.device_id}</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Channel:</span>
              <span className="text-white font-mono">{learnState.channel}</span>
            </div>
            <div className="flex justify-between text-xs">
              <span className="text-slate-400">Parameter:</span>
              <span className="text-white capitalize">{learnState.parameter_type}</span>
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex justify-between text-xs text-slate-400">
              <span>Move a MIDI control...</span>
              <span>{remainingSeconds.toFixed(1)}s</span>
            </div>
            <div className="w-full bg-slate-700 rounded-full h-2 overflow-hidden">
              <div
                className="bg-amber-500 h-full transition-all duration-100 ease-linear"
                style={{ width: `${progressPercent}%` }}
              />
            </div>
          </div>

          <p className="text-xs text-slate-500 mt-3 text-center">
            Press <kbd className="px-1.5 py-0.5 bg-slate-700 rounded text-slate-300">ESC</kbd> to
            cancel
          </p>
        </>
      )}
    </div>
  );
}
