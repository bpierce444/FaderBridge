import { useRef, useState, useCallback } from 'react';
import { motion } from 'framer-motion';
import { ActivityLight } from './ActivityLight';

export interface PanKnobProps {
  /** Current value (0.0 = full left, 0.5 = center, 1.0 = full right) */
  value: number;
  /** Callback when value changes */
  onChange: (value: number) => void;
  /** Label for the knob */
  label: string;
  /** Whether to show activity indicator */
  isActive?: boolean;
  /** Size of the knob in pixels (default: 64) */
  size?: number;
  /** Whether the knob is disabled */
  disabled?: boolean;
}

/**
 * Formats pan value for display
 * 0.0 = "L100", 0.5 = "C", 1.0 = "R100"
 */
function formatPan(value: number): string {
  if (Math.abs(value - 0.5) < 0.01) return 'C';
  if (value < 0.5) {
    const percent = Math.round((0.5 - value) * 200);
    return `L${percent}`;
  }
  const percent = Math.round((value - 0.5) * 200);
  return `R${percent}`;
}

/**
 * PanKnob component with rotation interaction.
 * Supports mouse drag and keyboard control.
 */
export function PanKnob({
  value,
  onChange,
  label,
  isActive = false,
  size = 64,
  disabled = false,
}: PanKnobProps) {
  const knobRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStartY, setDragStartY] = useState(0);
  const [dragStartValue, setDragStartValue] = useState(0);

  const handlePointerDown = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (disabled) return;

      setIsDragging(true);
      setDragStartY(e.clientY);
      setDragStartValue(value);
      e.currentTarget.setPointerCapture(e.pointerId);
    },
    [disabled, value]
  );

  const handlePointerMove = useCallback(
    (e: React.PointerEvent<HTMLDivElement>) => {
      if (!isDragging || disabled) return;

      const deltaY = dragStartY - e.clientY;
      const sensitivity = 0.005; // Adjust for desired rotation speed
      const newValue = Math.max(0, Math.min(1, dragStartValue + deltaY * sensitivity));
      onChange(newValue);
    },
    [isDragging, disabled, dragStartY, dragStartValue, onChange]
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
      if (e.key === 'ArrowRight' || e.key === 'ArrowUp') delta = 0.05;
      else if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') delta = -0.05;
      else if (e.key === 'Home') {
        onChange(0);
        return;
      } else if (e.key === 'End') {
        onChange(1);
        return;
      } else if (e.key === ' ') {
        e.preventDefault();
        onChange(0.5); // Center on spacebar
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

  // Calculate rotation angle (-135° to +135°)
  const rotation = (value - 0.5) * 270;

  const displayValue = formatPan(value);

  return (
    <div className="flex flex-col items-center gap-2 select-none">
      {/* Activity Light */}
      <ActivityLight active={isActive} color="cyan" size={6} ariaLabel={`${label} activity`} />

      {/* Knob */}
      <div
        ref={knobRef}
        className={`relative rounded-full bg-slate-800 cursor-pointer ${
          disabled ? 'opacity-50 cursor-not-allowed' : ''
        } ${isActive ? 'ring-2 ring-cyan-500/50' : ''}`}
        style={{ width: size, height: size }}
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
        {/* Knob Body */}
        <motion.div
          className={`absolute inset-0 rounded-full ${
            isActive
              ? 'bg-gradient-to-br from-cyan-400 to-cyan-600'
              : 'bg-gradient-to-br from-slate-500 to-slate-700'
          }`}
          animate={{
            rotate: rotation,
            boxShadow: isActive
              ? '0 0 20px rgba(6, 182, 212, 0.5)'
              : '0 4px 6px rgba(0, 0, 0, 0.3)',
          }}
          transition={{ duration: 0.15 }}
        >
          {/* Indicator Line */}
          <div className="absolute top-2 left-1/2 w-0.5 h-4 -ml-0.25 bg-white rounded-full" />

          {/* Center Dot */}
          <div className="absolute top-1/2 left-1/2 w-2 h-2 -ml-1 -mt-1 bg-slate-900 rounded-full" />
        </motion.div>

        {/* Tick Marks */}
        <div className="absolute inset-0 pointer-events-none">
          {/* Center tick */}
          <div
            className="absolute top-0 left-1/2 w-0.5 h-1 -ml-0.25 bg-slate-600"
            style={{ transform: 'translateY(-2px)' }}
          />
          {/* Left tick */}
          <div
            className="absolute top-1/2 left-0 w-1 h-0.5 -mt-0.25 bg-slate-600"
            style={{ transform: 'translateX(-2px)' }}
          />
          {/* Right tick */}
          <div
            className="absolute top-1/2 right-0 w-1 h-0.5 -mt-0.25 bg-slate-600"
            style={{ transform: 'translateX(2px)' }}
          />
        </div>
      </div>

      {/* Value Display */}
      <div
        className="font-mono text-sm text-slate-300 min-w-[3rem] text-center"
        aria-live="polite"
        aria-atomic="true"
      >
        {displayValue}
      </div>

      {/* Label */}
      <div className="text-xs text-slate-400 text-center max-w-[4rem] truncate" title={label}>
        {label}
      </div>
    </div>
  );
}
