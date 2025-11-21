import { useRef, useState, useCallback } from 'react';
import { motion } from 'framer-motion';
import { ActivityLight } from './ActivityLight';

export interface FaderProps {
  /** Current value (0.0 to 1.0) */
  value: number;
  /** Callback when value changes */
  onChange: (value: number) => void;
  /** Label for the fader */
  label: string;
  /** Whether to show activity indicator */
  isActive?: boolean;
  /** Display value as dB (default: true) */
  showDb?: boolean;
  /** Minimum dB value for display (default: -60) */
  minDb?: number;
  /** Maximum dB value for display (default: 10) */
  maxDb?: number;
  /** Height of the fader track in pixels (default: 200) */
  height?: number;
  /** Whether the fader is disabled */
  disabled?: boolean;
}

/**
 * Converts a normalized value (0-1) to dB scale
 */
function valueToDb(value: number, minDb: number, maxDb: number): number {
  if (value === 0) return minDb;
  return minDb + value * (maxDb - minDb);
}

/**
 * Formats dB value for display
 */
function formatDb(db: number): string {
  if (db <= -60) return '-∞';
  return db >= 0 ? `+${db.toFixed(1)}` : db.toFixed(1);
}

/**
 * Fader component with smooth drag interaction and visual feedback.
 * Supports both mouse drag and click-to-set interaction.
 */
export function Fader({
  value,
  onChange,
  label,
  isActive = false,
  showDb = true,
  minDb = -60,
  maxDb = 10,
  height = 200,
  disabled = false,
}: FaderProps) {
  const trackRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);

  const handlePointerDown = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (disabled) return;

      setIsDragging(true);
      e.currentTarget.setPointerCapture(e.pointerId);

      const updateValue = (clientY: number) => {
        if (!trackRef.current) return;

        const rect = trackRef.current.getBoundingClientRect();
        const y = clientY - rect.top;
        const newValue = 1 - Math.max(0, Math.min(1, y / rect.height));
        onChange(newValue);
      };

      updateValue(e.clientY);
    },
    [disabled, onChange]
  );

  const handlePointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!isDragging || disabled || !trackRef.current) return;

      const rect = trackRef.current.getBoundingClientRect();
      const y = e.clientY - rect.top;
      const newValue = 1 - Math.max(0, Math.min(1, y / rect.height));
      onChange(newValue);
    },
    [isDragging, disabled, onChange]
  );

  const handlePointerUp = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      setIsDragging(false);
      e.currentTarget.releasePointerCapture(e.pointerId);
    },
    []
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (disabled) return;

      let delta = 0;
      if (e.key === 'ArrowUp') delta = 0.05;
      else if (e.key === 'ArrowDown') delta = -0.05;
      else if (e.key === 'PageUp') delta = 0.1;
      else if (e.key === 'PageDown') delta = -0.1;
      else if (e.key === 'Home') {
        onChange(1);
        return;
      } else if (e.key === 'End') {
        onChange(0);
        return;
      }

      if (delta !== 0) {
        e.preventDefault();
        const newValue = Math.max(0, Math.min(1, value + delta));
        onChange(newValue);
      }
    },
    [disabled, value, onChange]
  );

  const displayValue = showDb
    ? formatDb(valueToDb(value, minDb, maxDb))
    : `${Math.round(value * 100)}%`;

  const capPosition = (1 - value) * height;

  return (
    <div className="flex flex-col items-center gap-2 select-none">
      {/* Activity Light */}
      <ActivityLight active={isActive} color="emerald" size={6} ariaLabel={`${label} activity`} />

      {/* Value Display */}
      <div
        className="font-mono text-sm text-slate-300 min-w-[4rem] text-center"
        aria-live="polite"
        aria-atomic="true"
      >
        {displayValue}
      </div>

      {/* Fader Track */}
      <div
        ref={trackRef}
        className={`relative w-12 bg-slate-800 rounded-full cursor-pointer ${
          disabled ? 'opacity-50 cursor-not-allowed' : ''
        } ${isActive ? 'ring-2 ring-emerald-500/50' : ''}`}
        style={{ height }}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={handlePointerUp}
        role="slider"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(value * 100)}
        aria-valuetext={displayValue}
        aria-disabled={disabled}
        tabIndex={disabled ? -1 : 0}
        onKeyDown={handleKeyDown}
      >
        {/* Fill */}
        <div
          className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-cyan-500 to-cyan-400 rounded-full transition-colors"
          style={{ height: `${value * 100}%` }}
        />

        {/* Fader Cap */}
        <motion.div
          className={`absolute left-1/2 w-14 h-8 -ml-7 rounded-lg shadow-lg cursor-grab active:cursor-grabbing ${
            isActive
              ? 'bg-gradient-to-b from-cyan-400 to-cyan-500 shadow-cyan-500/50'
              : 'bg-gradient-to-b from-slate-400 to-slate-500'
          }`}
          style={{
            top: capPosition - 16, // Center the cap on the position
          }}
          animate={{
            boxShadow: isActive
              ? '0 0 20px rgba(6, 182, 212, 0.5)'
              : '0 4px 6px rgba(0, 0, 0, 0.3)',
          }}
          transition={{ duration: 0.15 }}
        >
          {/* Grip lines */}
          <div className="flex items-center justify-center h-full gap-1">
            <div className="w-0.5 h-4 bg-slate-700 rounded-full" />
            <div className="w-0.5 h-4 bg-slate-700 rounded-full" />
            <div className="w-0.5 h-4 bg-slate-700 rounded-full" />
          </div>
        </motion.div>
      </div>

      {/* Label */}
      <div className="text-xs text-slate-400 text-center max-w-[4rem] truncate" title={label}>
        {label}
      </div>
    </div>
  );
}
