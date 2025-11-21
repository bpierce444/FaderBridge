/**
 * LearnButton component
 * A button that triggers MIDI Learn mode for a specific parameter
 */

import { useState } from 'react';
import { UcNetParameterType } from '../types/mapping';

export interface LearnButtonProps {
  /** UCNet device ID */
  deviceId: string;
  /** UCNet channel number */
  channel: number;
  /** Parameter type to learn */
  parameterType: UcNetParameterType;
  /** Whether currently in learn mode for this parameter */
  isLearning: boolean;
  /** Callback when learn mode is requested */
  onLearnStart: (deviceId: string, channel: number, parameterType: UcNetParameterType) => void;
  /** Callback when learn mode is cancelled */
  onLearnCancel: () => void;
  /** Optional CSS class name */
  className?: string;
}

/**
 * Button component for triggering MIDI Learn mode
 * 
 * @example
 * ```tsx
 * <LearnButton
 *   deviceId="device-1"
 *   channel={1}
 *   parameterType="volume"
 *   isLearning={false}
 *   onLearnStart={handleLearnStart}
 *   onLearnCancel={handleLearnCancel}
 * />
 * ```
 */
export function LearnButton({
  deviceId,
  channel,
  parameterType,
  isLearning,
  onLearnStart,
  onLearnCancel,
  className = '',
}: LearnButtonProps) {
  const [isHovered, setIsHovered] = useState(false);

  const handleClick = () => {
    if (isLearning) {
      onLearnCancel();
    } else {
      onLearnStart(deviceId, channel, parameterType);
    }
  };

  const baseClasses = 'px-3 py-1 rounded text-xs font-medium transition-all duration-200';
  const normalClasses = 'bg-slate-700 text-slate-300 hover:bg-slate-600 hover:text-white';
  const learningClasses = 'bg-amber-500 text-slate-900 animate-pulse hover:bg-amber-400';
  const stateClasses = isLearning ? learningClasses : normalClasses;

  return (
    <button
      onClick={handleClick}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      className={`${baseClasses} ${stateClasses} ${className}`}
      title={isLearning ? 'Click to cancel (or press ESC)' : 'Click to learn MIDI mapping'}
      aria-label={
        isLearning
          ? `Cancel MIDI Learn for ${parameterType} on channel ${channel}`
          : `Start MIDI Learn for ${parameterType} on channel ${channel}`
      }
    >
      {isLearning ? (
        <span className="flex items-center gap-1">
          <svg
            className="w-3 h-3 animate-spin"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          Listening...
        </span>
      ) : (
        <span className="flex items-center gap-1">
          <svg
            className="w-3 h-3"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15.536a5 5 0 001.414 1.414m2.828-9.9a9 9 0 012.828 2.828"
            />
          </svg>
          {isHovered ? 'Learn' : 'L'}
        </span>
      )}
    </button>
  );
}
