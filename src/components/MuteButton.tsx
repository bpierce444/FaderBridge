import { motion } from 'framer-motion';
import { ActivityLight } from './ActivityLight';

export interface MuteButtonProps {
  /** Whether the channel is muted */
  muted: boolean;
  /** Callback when mute state changes */
  onToggle: () => void;
  /** Label for the button */
  label?: string;
  /** Whether to show activity indicator */
  isActive?: boolean;
  /** Whether the button is disabled */
  disabled?: boolean;
}

/**
 * MuteButton component with toggle state and visual feedback.
 * Shows red when muted, slate when unmuted.
 */
export function MuteButton({
  muted,
  onToggle,
  label = 'Mute',
  isActive = false,
  disabled = false,
}: MuteButtonProps) {
  const handleClick = () => {
    if (!disabled) {
      onToggle();
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onToggle();
    }
  };

  return (
    <div className="flex flex-col items-center gap-2">
      {/* Activity Light */}
      <ActivityLight active={isActive} color="amber" size={6} ariaLabel={`${label} activity`} />

      {/* Button */}
      <motion.button
        onClick={handleClick}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        className={`
          relative w-16 h-16 rounded-lg font-bold text-sm
          transition-colors duration-150
          focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-950
          ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
          ${
            muted
              ? 'bg-rose-600 text-white focus:ring-rose-500'
              : 'bg-slate-700 text-slate-300 hover:bg-slate-600 focus:ring-slate-500'
          }
        `}
        whileTap={disabled ? {} : { scale: 0.95 }}
        animate={{
          boxShadow: isActive
            ? muted
              ? '0 0 20px rgba(225, 29, 72, 0.5)'
              : '0 0 20px rgba(6, 182, 212, 0.3)'
            : '0 4px 6px rgba(0, 0, 0, 0.3)',
        }}
        role="switch"
        aria-checked={muted}
        aria-label={label}
        aria-disabled={disabled}
      >
        <span className="uppercase tracking-wide">M</span>
        {muted && (
          <motion.div
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            className="absolute inset-0 flex items-center justify-center"
          >
            <div className="w-12 h-0.5 bg-white rotate-45 absolute" />
            <div className="w-12 h-0.5 bg-white -rotate-45 absolute" />
          </motion.div>
        )}
      </motion.button>

      {/* Label */}
      <div className="text-xs text-slate-400 text-center">{label}</div>
    </div>
  );
}
